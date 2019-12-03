use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result, Parameter
};
use sr_primitives::{
	traits::{
		SimpleArithmetic, Member, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, StaticLookup,
	},
};
use codec::Codec;
use system::ensure_signed;
use crate::traits::{Token, MintableToken};

pub trait Trait: system::Trait {
	type Balance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + MaybeSerializeDeserialize;

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as BDT {
		Init get(is_init): bool;
		Owner get(owner) config(): T::AccountId;
		Circulation get(circulation) config(): T::Balance;
		BalanceOf get(balance_of): map T::AccountId => T::Balance;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn init(origin) -> Result{
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_init() == false, "Already initialized.");
			ensure!(Self::owner() == sender, "Only owner can initalize.");

			<BalanceOf<T>>::insert(sender.clone(), Self::circulation());
			Init::put(true);

			Ok(())
		}

		pub fn transfer(origin, receiver: <T::Lookup as StaticLookup>::Source, #[compact] value: T::Balance) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<BalanceOf<T>>::exists(sender.clone()), "Account does not own this token.");
			let receiver = T::Lookup::lookup(receiver)?;

			<Self as Token<_>>::transfer(&sender, &receiver, value)
		}

		pub fn mint(origin, to: <T::Lookup as StaticLookup>::Source, #[compact] value: T::Balance) -> Result {
			let transactor = ensure_signed(origin)?;
			ensure!(transactor == Self::owner(), "only owner can use!");
			let to = T::Lookup::lookup(to)?;

			<Self as MintableToken<_>>::mint(&to, value)
		}

		pub fn burn(origin, from: <T::Lookup as StaticLookup>::Source, #[compact] value: T::Balance) -> Result {
			let transactor = ensure_signed(origin)?;
			ensure!(transactor == Self::owner(), "only owner can use!");
			let from = T::Lookup::lookup(from)?;

			<Self as MintableToken<_>>::burn(&from, value)
		}
	}
}

impl<T: Trait> Token<T::AccountId> for Module<T> {
	type Balance = T::Balance;

	fn total_supply() -> Self::Balance {
		Self::circulation()
	}

	fn balance_of(who: &T::AccountId) -> Self::Balance {
		<BalanceOf<T>>::get(who)
	}

	fn transfer(source: &T::AccountId, dest: &T::AccountId, value: Self::Balance) -> Result {
		let sender_balance = Self::balance_of(source);
		ensure!(sender_balance >= value, "Not enough balance.");
		let updated_sender_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;

		let receiver_balance = Self::balance_of(dest);
		let updated_receiver_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;

		// reduce sender balance
		<BalanceOf<T>>::insert(source, updated_sender_balance);
		// add receiver balance
		<BalanceOf<T>>::insert(dest, updated_receiver_balance);

		Self::deposit_event(RawEvent::Transfer(source.clone(), dest.clone(), value));
		Ok(())
	}
}

impl<T: Trait> MintableToken<T::AccountId> for Module<T> {
	fn mint(to: &T::AccountId, value: Self::Balance) -> Result {
		let to_balance = Self::balance_of(to);
		let updated_to_balance = to_balance.checked_add(&value).ok_or("overflow in balance")?;
		<BalanceOf<T>>::insert(to, updated_to_balance);

		let base_circulation = Self::circulation();
		let updated_circulation = base_circulation.checked_add(&value).ok_or("overflow in circulation")?;
		<Circulation<T>>::put(updated_circulation);

		Self::deposit_event(RawEvent::Mint(to.clone(), value));
		Ok(())
	}

	fn burn(from: &T::AccountId, value: Self::Balance) -> Result {
		let from_balance = Self::balance_of(from.clone());
		ensure!(from_balance >= value, "Not enough balance.");
		let updated_from_balance = from_balance.checked_sub(&value).ok_or("overflow in balance")?;
		<BalanceOf<T>>::insert(from.clone(), updated_from_balance);

		let base_circulation = Self::circulation();
		let updated_circulation = base_circulation.checked_sub(&value).ok_or("overflow in circulation")?;
		<Circulation<T>>::put(updated_circulation);

		Self::deposit_event(RawEvent::Burn(from.clone(), value));
		Ok(())
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
		Balance = <T as Trait>::Balance
	{
		Transfer(AccountId, AccountId, Balance),
		Mint(AccountId, Balance),
		Burn(AccountId, Balance),
	}
);
