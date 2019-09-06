
use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result
};
use system::ensure_signed;

/// The module's configuration trait.
pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
	trait Store for Module<T: Trait> as TokenModule {
		TotalSupply get(total_supply): u64 = 21000000; // deprecated
		Circulation get(circulation): u64 = 0;
		Admin get(admin): T::AccountId;

		BalanceOf get(balance_of): map T::AccountId => u64;
	}
}

// The module's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		fn init(origin) -> Result{
			let sender = ensure_signed(origin)?;
			<Admin<T>>::put(&sender);

			Ok(())
		}

		fn mint(origin, to: T::AccountId, value: u64) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::admin(), "only owner can use!");

			let receiver_balance = Self::balance_of(to.clone());
			let updated_to_balance = receiver_balance.checked_add(value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_add(value).ok_or("overflow in circulation")?;
			Circulation::put(updated_circulation);

			Self::deposit_event(RawEvent::Mint(to, value));

			Ok(())
		}

		fn burn(origin, to: T::AccountId, value:u64) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(sender == Self::admin(), "only owner can use!");

			let sender_balance = Self::balance_of(to.clone());
			ensure!(sender_balance >= value, "Not enough balance.");
			let updated_from_balance = sender_balance.checked_sub(value).ok_or("overflow in balance")?;
			<BalanceOf<T>>::insert(to.clone(), updated_from_balance);

			let base_circulation = Self::circulation();
			let updated_circulation = base_circulation.checked_sub(value).ok_or("overflow in circulation")?;
			Circulation::put(updated_circulation);

			Self::deposit_event(RawEvent::Burn(to, value));

			Ok(())
		}

		pub fn transfer(origin, to: T::AccountId, value: u64) -> Result {
			let sender = ensure_signed(origin)?;
			let sender_balance = Self::balance_of(sender.clone());
			ensure!(sender_balance >= value, "Not enough balance.");

			let updated_from_balance = sender_balance.checked_sub(value).ok_or("overflow in calculating balance")?;
			let receiver_balance = Self::balance_of(to.clone());
			let updated_to_balance = receiver_balance.checked_add(value).ok_or("overflow in calculating balance")?;

			// 发送者减少余额
			<BalanceOf<T>>::insert(sender.clone(), updated_from_balance);
			// 接受者增加余额
			<BalanceOf<T>>::insert(to.clone(), updated_to_balance);

			Self::deposit_event(RawEvent::Transfer(sender, to, value));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		Mint(AccountId, u64),
		Burn(AccountId, u64),
		Transfer(AccountId, AccountId, u64),
	}
);