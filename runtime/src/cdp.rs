use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result,
};
use sr_primitives::traits::{
	CheckedAdd, CheckedSub, Zero, SaturatedConversion
};
use codec::{Encode, Decode};
use system::ensure_signed;
use crate::traits::{Token, MintableToken};
use rstd::convert::TryInto;
use runtime_io;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	type Sai: MintableToken<Self::AccountId>; // Stable coin
	type Skr: Token<Self::AccountId>; // Abstracted collateral

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

type SaiBalanceOf<T> = <<T as Trait>::Sai as Token<<T as system::Trait>::AccountId>>::Balance;
type SkrBalanceOf<T> = <<T as Trait>::Skr as Token<<T as system::Trait>::AccountId>>::Balance;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct Cup<SkrBalance, SaiBalance> {
	pub id: u64,

	pub locked_collaterals: SkrBalance,
	pub debts: SaiBalance,
	pub art: SaiBalance,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Cdp {
		Owner get(owner) config(): T::AccountId;

		SkrPrice get(skr_price): u64 = 1;

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

		pub fn open(origin) -> Result {
			let sender = ensure_signed(origin)?;

			let all_cups_count = Self::all_cups_count();
			let new_all_cups_count = all_cups_count.checked_add(1).ok_or("Overflow adding a new cup")?;
			let cup = Cup {
				id: all_cups_count,
				locked_collaterals: Zero::zero(),
				art: Zero::zero(),
				debts: Zero::zero(),
			};
			<AllCupsArray<T>>::insert(all_cups_count, cup);
			<AllCupsCount>::put(new_all_cups_count);

			<CupOwner<T>>::insert(all_cups_count, &sender);

			let owned_cup_count = Self::owned_cup_count(&sender);
			let new_owned_cup_count = owned_cup_count.checked_add(1).ok_or("Overflow adding a new cup to owned cups array")?;
			<OwnedCupsArray<T>>::insert((sender.clone(), owned_cup_count), all_cups_count);
			<OwnedCupsCount<T>>::insert(&sender, new_owned_cup_count);

			Ok(())
		}
		
		// sender lock his collaterals to cdp owner account
		pub fn lock(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let mut cup = Self::cup_by_index(cup_index);
			cup.locked_collaterals = cup.locked_collaterals.checked_add(&amount).ok_or("Overflow adding locked_collaterals")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			T::Skr::transfer(&sender, &Self::owner(), amount);
			Ok(())
		}

		// sender take his collaterals
		pub fn free(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let cup_owner = Self::owner_of(cup_index).unwrap();
			ensure!(sender == cup_owner, "only cup owner can do it!");

			let mut cup = Self::cup_by_index(cup_index);

			cup.locked_collaterals = cup.locked_collaterals.checked_sub(&amount).unwrap_or(Zero::zero()); // check
			ensure!(Self::safe(&cup),  "Debt must keep safe!");
			<AllCupsArray<T>>::insert(cup_index, cup);

			T::Skr::transfer(&Self::owner(), &sender, amount);
			Ok(())
		}


		// release stable coin to sender
		pub fn draw(origin, owned_cup_index: u32, amount: SaiBalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let cup_owner = Self::owner_of(cup_index).unwrap();
			ensure!(sender == cup_owner, "only cup owner can do it!");

			let mut cup = Self::cup_by_index(cup_index);
			// TODO: add total debt record

			// TODO: art is tax
			// cup.art = cup.art.checked_add(&amount).ok_or("Overflow adding art")?;
			cup.debts = cup.debts.checked_add(&amount).ok_or("Overflow adding debts")?;
			ensure!(Self::safe(&cup),  "Debt must keep safe!");
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			// release stable coin
			T::Sai::mint(&sender, amount);

			Ok(())
		}

		pub fn wipe(origin, owned_cup_index: u32, amount: SaiBalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let mut cup = Self::cup_by_index(cup_index);
			cup.debts = cup.debts.checked_sub(&amount).ok_or("Overflow subing debts")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			T::Sai::burn(&sender, amount);

			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	fn safe(cup: &Cup<SkrBalanceOf<T>, SaiBalanceOf<T>>) -> bool {
		let skr_price = Self::skr_price();

		// let new_debts = cup.debts.checked_add(&amount).unwrap();
		let debts_u64 = cup.debts.saturated_into::<u64>();

		let locked_collaterals = cup.locked_collaterals.saturated_into::<u64>();
		let min_collateralization_ratio = 1.5_f64;
		let collateral_value = (locked_collaterals * skr_price).saturated_into::<u64>();
		let max_stablecoin_can_generate = ((collateral_value as f64) / min_collateralization_ratio) as u64;

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
