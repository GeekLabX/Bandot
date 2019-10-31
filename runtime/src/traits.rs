use rstd::result;
use codec::Codec;
use sr_primitives::traits::{SimpleArithmetic, MaybeSerializeDebug};

pub trait Token<AccountId> {
	type Balance: SimpleArithmetic + Codec + Copy + MaybeSerializeDebug + Default;

	fn total_supply() -> Self::Balance;
	fn balance_of(who: &AccountId) -> Self::Balance;
	fn transfer(source: &AccountId, dest: &AccountId, value: Self::Balance) -> result::Result<(), &'static str>;
}

pub trait MintableToken<AccountId>: Token<AccountId> {
	fn mint(to: &AccountId, value: Self::Balance) -> result::Result<(), &'static str>;
	fn burn(from: &AccountId, value: Self::Balance) -> result::Result<(), &'static str>;
}
