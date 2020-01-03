use frame_support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result,
};
// use support::traits::{
	// Currency
// };
use sp_primitives::traits::{
	CheckedAdd, Zero,
};
use codec::{Encode, Decode};
use system::ensure_signed;
use crate::traits::{Token, MintableToken};

/// The module's configuration trait.
pub trait Trait: system::Trait {
	type Sc: MintableToken<Self::AccountId>; // Stable coin
	type Coll: Token<Self::AccountId>; // Abstracted collateral
	// type Gem: Currency<Self::AccountId>; // Underlying collateral

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

type ScBalanceOf<T> = <<T as Trait>::Sc as Token<<T as system::Trait>::AccountId>>::Balance;
type CollBalanceOf<T> = <<T as Trait>::Coll as Token<<T as system::Trait>::AccountId>>::Balance;

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct UserAssets<T: Trait> {
	pub staking_amount: CollBalanceOf<T>,
	pub locked_amount: CollBalanceOf<T>,
	pub lending_amount: CollBalanceOf<T>,
}

impl<T: Trait> Default for UserAssets<T> {
	fn default() -> Self {
		UserAssets {
			staking_amount: Zero::zero(),
			locked_amount: Zero::zero(),
			lending_amount: Zero::zero(),
		}
	}
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as Pool {
		Owner get(owner) config(): T::AccountId;
		// Assets of current pooling
		PoolAssets get(pool_assets): CollBalanceOf<T> = Zero::zero();
		// Assets info of current user
		UserAssetsInfo get(user_assets_info): map T::AccountId => UserAssets<T>;
		// set rate fee by Oracle service
		RateFee1k get(rate_fee1k): u8 = 1;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		pub fn deposit(origin, #[compact] amount: CollBalanceOf<T>) -> Result {
			let owner = ensure_signed(origin)?;
			let mut user_asserts = Self::user_assets_info(owner.clone());

			let pre_staking_amount = user_asserts.staking_amount;

			user_asserts.staking_amount = pre_staking_amount.checked_add(&amount).ok_or("overflow in staking")?;
			<UserAssetsInfo<T>>::insert(owner.clone(), user_asserts);

			let pre_pool_assets = Self::pool_assets();
			let updated_pool_assets = pre_pool_assets.checked_add(&amount).ok_or("overflow in pool")?;
			<PoolAssets<T>>::put(updated_pool_assets);

			Self::deposit_event(RawEvent::Deposit(owner, amount, updated_pool_assets));
			Ok(())
		}

		pub fn exchange(origin, #[compact] amount: CollBalanceOf<T>) -> Result {
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
		Balance = CollBalanceOf<T>
	{
		
		Deposit(AccountId, Balance, Balance),
		Exchange(AccountId, Balance, u128),
	}
);
