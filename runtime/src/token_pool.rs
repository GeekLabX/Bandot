use support::{
	decl_module, decl_storage, decl_event, ensure,
	StorageValue, StorageMap, dispatch::Result, Parameter
};
use sr_primitives::{
	traits::{
		SimpleArithmetic, Member, CheckedAdd, CheckedSub, MaybeSerializeDebug,
	},
};
// use codec::{Encode, Decode, Codec};
// use system::ensure_signed;
use crate::token;

pub trait Trait: system::Trait {

}

decl_storage! {
	trait Store for Module<T: Trait> as TokenPool {

	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// fn deposit_event() = default;

		// fn 
	}
}

