use frame_support::{
	decl_module, decl_storage, decl_event, ensure
};
use sp_runtime::{
	traits::{CheckedAdd, CheckedSub, Zero, SaturatedConversion},
	RuntimeDebug, DispatchResult
};
use codec::{Encode, Decode};
use crate::traits::{Token, MintableToken};
use system::{self as system, ensure_signed, ensure_root};

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct ExchangeRateItem<AccountId, BlockNumber> {
	owner: AccountId,
	rate: u32,
	start_time: BlockNumber,
}

/// The module's configuration trait.
pub trait Trait: system::Trait + pallet_timestamp::Trait {
	type Bdt: MintableToken<Self::AccountId>; // Stable coin
	type Skr: Token<Self::AccountId>; // Abstracted collateral

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// bdt稳定币, skr抵押币
type BdtBalance<T> = <<T as Trait>::Bdt as Token<<T as system::Trait>::AccountId>>::Balance;
type SkrBalance<T> = <<T as Trait>::Skr as Token<<T as system::Trait>::AccountId>>::Balance;

// 抵押仓位 Position
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Pos<SkrBalance, BdtBalance> {
	pub id: u64,

	pub lock_skr: SkrBalance,
	pub lend_bdt: BdtBalance,
	pub fee_bdt: BdtBalance,
}

decl_storage! {
	trait Store for Module<T: Trait> as Cdp {
		Owner get(owner) config(): T::AccountId;

		SkrPrice get(skr_price): u64 = 1;
		// 150% -> 150000
		BdtFee get(stability_fee): u32 = 5_000;

		PosOwner get(owner_of): map u64 => Option<T::AccountId>;
		PosCount get(pos_count): u64; /// 总Pos
		PosArray get(pos_index): map u64 => Pos<SkrBalance<T>, BdtBalance<T>>; /// 总 i => Pos 映射
		OwnedPosCount get(owned_pos_count): map T::AccountId => u64; // 拥有Pos
		OwnedPosArray get(owned_pos_index): map (T::AccountId, u64) => u64; // 拥有 i => PosArray index 映射
		
		ExchangeRates get(fn exchange_rates): map T::AccountId =>
				Option<ExchangeRateItem<T::AccountId, T::BlockNumber>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn update_price(origin, price: u64) {
			ensure_root(origin)?;
			SkrPrice::put(price);
		}

		// cdp 开仓
		pub fn open(origin) -> DispatchResult{
			let sender = ensure_signed(origin)?;

			let pos_count = Self::pos_count();
			let new_pos_count = pos_count.checked_add(1).ok_or("Overflow New Pos")?;
			let pos = Pos {
				id: pos_count,
				lock_skr: Zero::zero(),
				lend_bdt: Zero::zero(),
				fee_bdt: Zero::zero(),
			};
			// 修改 last index =>新 Pos
			<PosArray<T>>::insert(pos_count, pos);
			<PosCount>::put(new_pos_count);

			// 修改 index => 开仓的人
			<PosOwner<T>>::insert(pos_count, &sender);
			let owned_pos_count = Self::owned_pos_count(&sender);
			let new_owned_pos_count = owned_pos_count.checked_add(1).ok_or("Overflow Owned Pos")?;
			// owned index => pos index
			<OwnedPosArray<T>>::insert((sender.clone(), owned_pos_count), pos_count);
			<OwnedPosCount<T>>::insert(&sender, new_owned_pos_count);

			Self::deposit_event(RawEvent::Open(sender));
			Ok(())
		}
		
		// lock 锁定
		pub fn lock(origin, owned_index: u64, amount: SkrBalance<T>) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			// 取出 Pos
			ensure!(owned_index < Self::owned_pos_count(&sender), "owned_index too big!");
			let pos_index = Self::owned_pos_index((sender.clone(), owned_index));
			let mut pos = Self::pos_index(pos_index);
			
			pos.lock_skr = pos.lock_skr.checked_add(&amount).ok_or("Overflow add lock_skr")?;
			<PosArray<T>>::insert(pos_index, pos);
			// 这里Skr是要之前先发行
			T::Skr::transfer(&sender, &Self::owner(), amount);
			Self::deposit_event(RawEvent::Lock(sender, owned_index, amount));
			Ok(())
		}

		// free 解锁lock_skr
		pub fn free(origin, owned_index: u64, amount: SkrBalance<T>) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			// 取出Pos
			ensure!(owned_index < Self::owned_pos_count(&sender), "owned_index too big!");
			let pos_index = Self::owned_pos_index((sender.clone(), owned_index));
			let mut pos = Self::pos_index(pos_index);
			// 判断减法
			ensure!(amount < pos.lock_skr, "amount must be less than lock_skr!");
			// 解锁必须判断是否少于抵押率
			let lock_skr = pos.lock_skr.checked_sub(&amount).ok_or("Overflow sub lock_skr")?; 
			let lend_bdt = pos.lend_bdt;
			ensure!(Self::safe(lock_skr, lend_bdt),  "Debt must safe!");
			pos.lock_skr = lock_skr;
			<PosArray<T>>::insert(pos_index, pos);
			// 发出Skr
			T::Skr::transfer(&Self::owner(), &sender, amount);
			Self::deposit_event(RawEvent::Free(sender, owned_index, amount));
			Ok(())
		}


		// draw 借出
		pub fn draw(origin, owned_index: u64, amount: BdtBalance<T>) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			// 取出 Pos
			ensure!(owned_index < Self::owned_pos_count(&sender), "owned_index too big!");
			let pos_index = Self::owned_pos_index((sender.clone(), owned_index));
			let mut pos = Self::pos_index(pos_index);

			// 解锁必须判断是否少于抵押率
			let lock_skr = pos.lock_skr;
			let lend_bdt = pos.lend_bdt.checked_add(&amount).ok_or("Overflow add lend_bdt")?;
			ensure!(Self::safe(lock_skr, lend_bdt),  "Debt must safe!");	
			pos.lend_bdt =lend_bdt;		
			<PosArray<T>>::insert(pos_index, pos);
			// 产币
			T::Bdt::mint(&sender, amount);
			Self::deposit_event(RawEvent::Draw(sender, owned_index, amount));
			Ok(())
		}

		// wipe 还债
		pub fn wipe(origin, owned_index: u64, amount: BdtBalance<T>) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			// 取出 Pos
			ensure!(owned_index < Self::owned_pos_count(&sender), "owned_index too big!");
			let pos_index = Self::owned_pos_index((sender.clone(), owned_index));
			let mut pos = Self::pos_index(pos_index);
			
			// 减必须判断
			ensure!(amount < pos.lend_bdt, "amount must be less than lend_bdt!");
			pos.lend_bdt = pos.lend_bdt.checked_sub(&amount).ok_or("Overflow sub lend_bdt")?;
			<PosArray<T>>::insert(pos_index, pos);
			// 销毁
			T::Bdt::burn(&sender, amount);
			Self::deposit_event(RawEvent::Wipe(sender, owned_index, amount));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn safe(lock_skr: SkrBalance<T>, lend_bdt: BdtBalance<T>) -> bool {
		let skr_price = Self::skr_price();

		// 债务
		let lock_u64 = lock_skr.saturated_into::<u64>();
		let lend_u64 = lend_bdt.saturated_into::<u64>();

		let min_ratio = 1.5_f64;
		let lock_value = (lock_u64 * skr_price).saturated_into::<u64>();
		let blow_value = ((lend_u64 as f64) * min_ratio) as u64;

		// 债务不能大于最大可贷的稳定币量
		return blow_value <= lock_value
	}
}

decl_event!(
	pub enum Event<T> 
	where 
	    AccountId = <T as system::Trait>::AccountId,
		SkrBalance = SkrBalance<T>,
		BdtBalance = BdtBalance<T>
	    {
			Open(AccountId),
			Lock(AccountId, u64, SkrBalance),
			Free(AccountId, u64, SkrBalance),
			Draw(AccountId, u64, BdtBalance),
			Wipe(AccountId, u64, BdtBalance),

	    }
);

