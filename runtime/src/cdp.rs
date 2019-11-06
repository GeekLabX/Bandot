use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result,
};
use sr_primitives::traits::{
	CheckedAdd, Zero,
};
use codec::{Encode, Decode};
use system::ensure_signed;
use crate::traits::{Token, MintableToken};
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
pub struct Cup<AccountId, SkrBalance, SaiBalance> {
	pub lad: AccountId,
	pub ink: SkrBalance,
	pub art: SaiBalance,
	pub ire: SaiBalance,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Cdp {
		Owner get(owner) config(): T::AccountId;
		Cupi get(cupi): u32;
		Cups get(cups): map u32 => Cup<T::AccountId, SkrBalanceOf<T>, SaiBalanceOf<T>>;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn open(origin) -> Result {
			let owner = ensure_signed(origin)?;
			let cupi = Self::cupi();
			let new_cupi = cupi.checked_add(1).ok_or("Overflow adding a new cup")?;
			Cupi::put(new_cupi);
			let new_cup = Cup {
				lad: owner,
				ink: Zero::zero(),
				art: Zero::zero(),
				ire: Zero::zero(),
			};
			<Cups<T>>::insert(new_cupi, new_cup);
			Ok(())
		}

		pub fn lock(origin, cupi: u32, amount: SkrBalanceOf<T>) -> Result {
			let transactor = ensure_signed(origin)?;
			let mut cup = <Cups<T>>::get(cupi);
			cup.ink = cup.ink.checked_add(&amount).unwrap();
			T::Skr::transfer(&transactor, &Self::owner(), amount);
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
