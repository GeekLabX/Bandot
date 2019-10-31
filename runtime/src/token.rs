use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result, Parameter
};
use sr_primitives::{
	traits::{
		SimpleArithmetic, Member, CheckedAdd, CheckedSub, MaybeSerializeDebug,
	},
};
use codec::{Encode, Decode, Codec};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: system::Trait {
	type TokenBalance: Parameter + Member + SimpleArithmetic + Codec + Default + Copy + MaybeSerializeDebug;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct UserAssets {
	pub staking_amount: u128,
	pub locked_amount: u128,
	pub lending_amount: u128,
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TokenModule {
		Init get(is_init): bool;
		Owner get(owner) config(): T::AccountId;
		// circulation
		Circulation get(circulation) config(): T::TokenBalance;
		// BDT balance of user
		BalanceOf get(balance_of): map T::AccountId => T::TokenBalance;
		// Assets of current pooling
		PoolAssets get(pool_assets): u128 = 0;
		// Assets info of current user
		UserAssetsInfo get(user_assets_info): map T::AccountId => UserAssets;
		// set rate fee by Oracle service
		RateFee1k get(rate_fee1k): u8 = 1;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn init(origin) -> Result{
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_init() == false, "Already initialized.");
			ensure!(Self::owner() == sender, "Only owner can initalize.");

			<BalanceOf<T>>::insert(sender.clone(), Self::circulation());
			Init::put(true);

			Ok(())
		}

		fn mint(origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::owner(), "only owner can use!");

			let receiver_balance = Self::balance_of(to.clone());
			let updated_to_balance = receiver_balance.checked_add(&value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_add(&value).ok_or("overflow in circulation")?;
			<Circulation<T>>::put(updated_circulation);

			Self::deposit_event(RawEvent::Mint(to, value));

			Ok(())
		}

		fn burn(origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::owner(), "only owner can use!");

			let sender_balance = Self::balance_of(to.clone());
			ensure!(sender_balance >= value, "Not enough balance.");
			let updated_from_balance = sender_balance.checked_sub(&value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_from_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_sub(&value).ok_or("overflow in circulation")?;
			<Circulation<T>>::put(updated_circulation);

			Self::deposit_event(RawEvent::Burn(to, value));

			Ok(())
		}

		pub fn transfer(origin, to: T::AccountId, #[compact] value: T::TokenBalance) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<BalanceOf<T>>::exists(sender.clone()), "Account does not own this token.");

			let sender_balance = Self::balance_of(sender.clone());
			ensure!(sender_balance >= value, "Not enough balance.");
			let updated_sender_balance = sender_balance.checked_sub(&value).ok_or("overflow in calculating balance")?;

			let receiver_balance = Self::balance_of(to.clone());
			let updated_receiver_balance = receiver_balance.checked_add(&value).ok_or("overflow in calculating balance")?;

			// reduce sender balance
			<BalanceOf<T>>::insert(sender.clone(), updated_sender_balance);
			// add receiver balance
			<BalanceOf<T>>::insert(to.clone(), updated_receiver_balance);

			Self::deposit_event(RawEvent::Transfer(sender, to, value));
			Ok(())
		}

		pub fn deposit(origin, amount: u128) -> Result {
			let owner = ensure_signed(origin)?;
			let mut user_asserts = Self::user_assets_info(owner.clone());

			let pre_staking_amount = user_asserts.staking_amount;

			user_asserts.staking_amount = pre_staking_amount.checked_add(amount).ok_or("overflow in staking")?;
			<UserAssetsInfo<T>>::insert(owner.clone(), user_asserts);

			let pre_pool_assets = Self::pool_assets();
			let updated_pool_assets = pre_pool_assets.checked_add(amount).ok_or("overflow in pool")?;
			PoolAssets::put(updated_pool_assets);

			Self::deposit_event(RawEvent::Deposit(owner, amount, updated_pool_assets));
			Ok(())
		}

		pub fn exchange(origin, amount: u128) -> Result {
			let owner = ensure_signed(origin)?;
			let mut user_asserts = Self::user_assets_info(owner.clone());

			// pre-exchange
			let pre_locked_amount = user_asserts.locked_amount;
			let pre_lending_amount = user_asserts.lending_amount;

			ensure!(pre_lending_amount >= pre_lending_amount + amount, "Not enough stake.");
			user_asserts.lending_amount = pre_lending_amount + amount;
			user_asserts.locked_amount = pre_locked_amount + amount;

			let fee = 1u128;
			<UserAssetsInfo<T>>::insert(owner.clone(), user_asserts);

			Self::deposit_event(RawEvent::Exchange(owner, amount, fee));
			Ok(())
		}

		pub fn set_fee(origin, fee: u8) -> Result {
			let sender = ensure_signed(origin)?;
            ensure!(sender == Self::owner(), "only owner can use!");

			RateFee1k::put(fee);
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> 
	where 
		AccountId = <T as system::Trait>::AccountId,
		Balance = <T as Trait>::TokenBalance
	{
		
		Mint(AccountId, Balance),
		Burn(AccountId, Balance),
		Transfer(AccountId, AccountId, Balance),
		Deposit(AccountId, u128, u128),
		Exchange(AccountId, u128, u128),
	}
);
