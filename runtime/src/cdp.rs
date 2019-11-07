use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result,
};
use sr_primitives::traits::{
	CheckedAdd, Zero, SaturatedConversion
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

	pub ink: SkrBalance,
	pub art: SaiBalance,
	pub ire: SaiBalance,
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
				ink: Zero::zero(),
				art: Zero::zero(),
				ire: Zero::zero(),
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
		
		// sender lock his pdot to cdp owner
		pub fn lock(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) -> Result {
			let transactor = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((transactor.clone(), owned_cup_index));
			let mut cup = Self::cup_by_index(cup_index);
			cup.ink = cup.ink.checked_add(&amount).ok_or("Overflow adding ink")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			T::Skr::transfer(&transactor, &Self::owner(), amount);
			Ok(())
		}

		pub fn free(origin, owned_cup_index: u32, amount: SkrBalanceOf<T>) -> Result {
			// TODO: free logic
			Ok(())
		}

		// release stable coin to sender
		pub fn draw(origin, owned_cup_index: u32, amount: SaiBalanceOf<T>) -> Result {
			let sender = ensure_signed(origin)?;
			let cup_index = Self::cup_of_owner_by_index((sender.clone(), owned_cup_index));
			let cup_owner = Self::owner_of(cup_index).unwrap();
			ensure!(sender == cup_owner, "only cup owner can do it!");

			let mut cup = Self::cup_by_index(cup_index);
			let new_ire = cup.ire.checked_add(&amount).ok_or("Overflow adding ire")?;
			let skr_price = Self::skr_price();
			// TODO: add total debt record
			// TODO: add debt ratio
			let new_ire_u64 = new_ire.saturated_into::<u64>();
			let ink_u64 = cup.ink.saturated_into::<u64>();
			let debt_cap_u64 = (ink_u64 * skr_price).saturated_into::<u64>();
			ensure!((new_ire_u64 <= debt_cap_u64), "Debt must keep safe!");
			// TODO: art is tax
			// cup.art = cup.art.checked_add(&amount).ok_or("Overflow adding art")?;
			cup.ire = cup.ire.checked_add(&amount).ok_or("Overflow adding ire")?;
			<AllCupsArray<T>>::insert(cup_index, cup);
			
			// release stable coin
			T::Sai::mint(&sender, amount);

			Ok(())
		}
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
