use rstd::fmt::Debug;
use sp_runtime::{
	traits::{SimpleArithmetic, MaybeSerializeDeserialize},
	DispatchResult
};
use codec::FullCodec;

pub trait Token<AccountId> {
	type Balance: SimpleArithmetic + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	fn total_supply() -> Self::Balance;
	fn balance_of(who: &AccountId) -> Self::Balance;
	fn transfer(source: &AccountId, dest: &AccountId, value: Self::Balance) -> DispatchResult;
}

pub trait MintableToken<AccountId>: Token<AccountId> {
	fn mint(to: &AccountId, value: Self::Balance) -> DispatchResult;
	fn burn(from: &AccountId, value: Self::Balance) -> DispatchResult;
}
