use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result, print, 
};
use sr_primitives::traits::{
	CheckedAdd, CheckedSub, Zero, SaturatedConversion
};
use codec::{Encode, Decode};
use system::{ensure_signed, ensure_root};
use crate::traits::{Token, MintableToken};
use rstd::convert::TryInto;
use arithmetic::Fixed64;
use runtime_io;

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
	type Sai: MintableToken<Self::AccountId>; // Stable coin
	type Skr: Token<Self::AccountId>; // Abstracted collateral

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

type SaiBalanceOf<T> = <<T as Trait>::Sai as Token<<T as system::Trait>::AccountId>>::Balance;
type SkrBalanceOf<T> = <<T as Trait>::Skr as Token<<T as system::Trait>::AccountId>>::Balance;

const DOTS: u128 = 1_000_000_000_000;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Cup<SkrBalance, SaiBalance> {
	pub id: u64,

	pub locked_collaterals: SkrBalance,
	pub debts: SaiBalance,
	pub tax: SaiBalance,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Cdp {
		Owner get(owner) config(): T::AccountId;

		SkrPrice get(skr_price): u64 = 1;
		// 150% -> 150000
		MinCollaterlizationRatio get(min_collateralization_ratio): u32 = 150_000;
		StabilityFee get(stability_fee): u32 = 5_000;

		CupOwner get(owner_of): map u64 => Option<T::AccountId>;
		AllCupsArray get(cup_by_index): map u64 => Cup<SkrBalanceOf<T>, SaiBalanceOf<T>>;
		AllCupsCount get(all_cups_count): u64;
		OwnedCupsArray get(cup_of_owner_by_index): map (T::AccountId, u32) => u64;
		OwnedCupsCount get(owned_cup_count): map T::AccountId => u32;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn update_skr_price(origin, price: u64) {
			ensure_root(origin)?;
			SkrPrice::put(price);
		}

		pub fn get_time(origin) -> Result {
			let _sender = ensure_signed(origin)?;
			let now = <timestamp::Module<T>>::get();
			Ok(())
		}

		// cdp开仓
		pub fn open(origin) {
			let sender = ensure_signed(origin)?;

			let all_cups_count = Self::all_cups_count();
			let new_all_cups_count = all_cups_count.checked_add(1).ok_or("Overflow adding a new cup")?;
			let cup = Cup {
				id: all_cups_count,
				locked_collaterals: Zero::zero(),
				debts: Zero::zero(),
				tax: Zero::zero(),
			};
			<AllCupsArray<T>>::insert(all_cups_count, cup);
			<AllCupsCount>::put(new_all_cups_count);

			<CupOwner<T>>::insert(all_cups_count, &sender);

			let owned_cup_count = Self::owned_cup_count(&sender);
			let new_owned_cup_count = owned_cup_count.checked_add(1).ok_or("Overflow adding a new cup to owned cups array")?;
			<OwnedCupsArray<T>>::insert((sender.clone(), owned_cup_count), all_cups_count);
			<OwnedCupsCount<T>>::insert(&sender, new_owned_cup_count);
		}
		
		// sender lock his collaterals to cdp owner account
		pub fn lock(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let mut cup = Self::cup_by_index(cup_index);
			cup.locked_collaterals = cup.locked_collaterals.checked_add(&amount).ok_or("Overflow adding locked_collaterals")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			T::Skr::transfer(&sender, &Self::owner(), amount);
		}

		// sender take his collaterals
		pub fn free(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let cup_owner = Self::owner_of(cup_index).unwrap();
			ensure!(sender == cup_owner, "only cup owner can do it!");

			let mut cup = Self::cup_by_index(cup_index);

			cup.locked_collaterals = cup.locked_collaterals.checked_sub(&amount).ok_or("Overflow subbing")?; // check
			ensure!(Self::safe(&cup),  "Debt must keep safe!");
			<AllCupsArray<T>>::insert(cup_index, cup);

			T::Skr::transfer(&Self::owner(), &sender, amount);
		}


		// release stable coin to sender
		pub fn draw(origin, owned_cup_index: u32, amount: SaiBalanceOf<T>) {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let cup_owner = Self::owner_of(cup_index).unwrap();
			ensure!(sender == cup_owner, "only cup owner can do it!");

			let mut cup = Self::cup_by_index(cup_index);
			// TODO: add total debt record

			// cup.tax = cup.tax.checked_add(&amount).ok_or("Overflow adding tax")?;
			cup.debts = cup.debts.checked_add(&amount).ok_or("Overflow adding debts")?;
			ensure!(Self::safe(&cup),  "Debt must keep safe!");
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			// release stable coin
			T::Sai::mint(&sender, amount);
		}

		// 还债
		pub fn wipe(origin, owned_cup_index: u32, amount: SaiBalanceOf<T>) {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let mut cup = Self::cup_by_index(cup_index);
			cup.debts = cup.debts.checked_sub(&amount).ok_or("Overflow subbing debts")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			T::Sai::burn(&sender, amount);
		}
	}
}

impl<T: Trait> Module<T> {
	fn safe(cup: &Cup<SkrBalanceOf<T>, SaiBalanceOf<T>>) -> bool {
		let skr_price = Self::skr_price();

		// 债务
		let debts_u64 = cup.debts.saturated_into::<u64>();

		// 最大可贷的稳定币量
		// (locked_collaterals * skr_price) / 1.5
		let locked_collaterals = cup.locked_collaterals.saturated_into::<u64>();
		let min_collateralization_ratio = 1.5_f64;
		let collateral_value = (locked_collaterals * skr_price).saturated_into::<u64>();
		let max_stablecoin_can_generate = ((collateral_value as f64) / min_collateralization_ratio) as u64;

		// 债务不能大于最大可贷的稳定币量
		return debts_u64 <= max_stablecoin_can_generate
	}
}

decl_event!(
	pub enum Event<T> 
	where 
	    AccountId = <T as system::Trait>::AccountId,
	    Balance = SkrBalanceOf<T>
	    {
		    NewCup(AccountId, Balance),
	    }
);

#[cfg(test)]
mod tests {
	use super::*;
	use crate::pdot;
	use crate::bdt;

	use sr_primitives::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, weights::Weight, Perbill,
	};

	use support::{assert_ok, impl_outer_origin, parameter_types};
	use primitives::H256;

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
		pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
	}
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Call = ();
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl bdt::Trait for Test {
		type Balance = u64;
		type Event = ();
	}
	impl pdot::Trait for Test {
		type Balance = u64;
		type Event = ();
	}
	impl Trait for Test {
		type Event = ();
		type Sai = Bdt;
		type Skr = Pdot;
	}
	type Cdp = Module<Test>;
	type Bdt = bdt::Module<Test>;
	type Pdot = pdot::Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities {
		let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		let _ = bdt::GenesisConfig::<Test> {
			owner: 1,
			circulation: 0,
		}.assimilate_storage(&mut t);
		let _ = pdot::GenesisConfig::<Test> {
			owner: 2,
			circulation: 0,
		}.assimilate_storage(&mut t);
		GenesisConfig::<Test>{
			owner: 3,
		}.assimilate_storage(&mut t).unwrap();
		t.into()
	}

	#[test]
	fn open_position_and_lock_works() {
		new_test_ext().execute_with(|| {
			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);
			assert_ok!(Cdp::open(Origin::signed(4)));
			assert_eq!(Cdp::all_cups_count(), 1);
			Cdp::lock(Origin::signed(4), 0, 999);
			assert_eq!(Pdot::balance_of(4), 9000);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 999);
 		});
 	}

	#[test]
	fn should_not_lock_more_than_account_balance() {
		new_test_ext().execute_with(|| {
			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);

			Cdp::open(Origin::signed(4));

			Cdp::lock(Origin::signed(4), 0, 999);
			assert_eq!(Pdot::balance_of(4), 9000);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 999);

			Cdp::lock(Origin::signed(4), 0, 10000);
			assert_eq!(Pdot::balance_of(4), 9000);
		});
	}

	#[test]
	fn free_works() {
		new_test_ext().execute_with(|| {
			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);

			Cdp::open(Origin::signed(4));
			Cdp::lock(Origin::signed(4), 0, 999);
			assert_eq!(Pdot::balance_of(4), 9000);

			Cdp::free(Origin::signed(4), 0, 100);
			assert_eq!(Pdot::balance_of(4), 9100);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 899);

			Cdp::free(Origin::signed(4), 0, 899);
			assert_eq!(Pdot::balance_of(4), 9999);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 0);
		});
	}

	#[test]
	fn should_not_free_more_than_locked() {
		new_test_ext().execute_with(|| {
			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);

			Cdp::open(Origin::signed(4));
			Cdp::lock(Origin::signed(4), 0, 999);
			assert_eq!(Pdot::balance_of(4), 9000);

			Cdp::free(Origin::signed(4), 0, 1000);
			assert_eq!(Pdot::balance_of(4), 9000);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 999);
		});
	}

	#[test]
	fn draw_works() {
		new_test_ext().execute_with(|| {
			Cdp::update_skr_price(Origin::ROOT, 1);

			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);

			Bdt::init(Origin::signed(2));
			assert_eq!(Bdt::balance_of(4), 0);

			Cdp::open(Origin::signed(4));
			Cdp::lock(Origin::signed(4), 0, 999);

			Cdp::draw(Origin::signed(4), 0, 100);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 999);
			assert_eq!(cdp.debts, 100);
			assert_eq!(Bdt::balance_of(4), 100);

			// 计算剩余能draw的稳定币
			let max_stablecoin_can_generate = ((cdp.locked_collaterals * Cdp::skr_price()) as f64 / 1.5) as u64;
			let left_stablecoin_can_generate = max_stablecoin_can_generate - 100;

			// 多一个都不能draw成功
			Cdp::draw(Origin::signed(4), 0, left_stablecoin_can_generate + 1);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.debts, 100);
		});
	}

	#[test]
	fn should_not_free_more_than_locked_sub_drawed() {
		new_test_ext().execute_with(|| {
			Cdp::update_skr_price(Origin::ROOT, 1);

			// 给用户4初始化9999个dot
			Pdot::init(Origin::signed(2));
			Pdot::mint(Origin::signed(2), 4, 9999);
			assert_eq!(Pdot::balance_of(4), 9999);

			// 给用户4初始化0个dot
			Bdt::init(Origin::signed(2));
			assert_eq!(Bdt::balance_of(4), 0);

			// 用户4开仓,然后押999个dot
			Cdp::open(Origin::signed(4));
			Cdp::lock(Origin::signed(4), 0, 999);
			// 用户4此时剩余了9000个dot
			assert_eq!(Pdot::balance_of(4), 9000);

			// 用户4从刚开仓的cdp中提走100个bdt
			Cdp::draw(Origin::signed(4), 0, 100);

			// 用户4拿回押着的849个dot，那么后面其实就剩150个dot在押
			Cdp::free(Origin::signed(4), 0, 849);
			// 用户4此时pdot余额是9849个
			assert_eq!(Pdot::balance_of(4), 9849);

			// 检查cdp，发现真到只有150个dot在押
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 150);

			// 计算剩余能free的dot
			let max_stablecoin_can_generate = ((150 * Cdp::skr_price()) as f64 / 1.5) as u64;
			assert_eq!(cdp.debts, 100); // 而目前债务就是100个bdt
			let left_stablecoin_can_generate = max_stablecoin_can_generate - 100;
			let left_skr_can_be_free = (left_stablecoin_can_generate as f64 / 1.5) as u64;

			// 加1都不行
			Cdp::free(Origin::signed(4), 0, left_skr_can_be_free + 1);
			assert_eq!(Pdot::balance_of(4), 9849);
			let cdp = Cdp::cup_by_index(0);
			assert_eq!(cdp.locked_collaterals, 150);
			assert_eq!(cdp.debts, 100);
		});
	}
}
