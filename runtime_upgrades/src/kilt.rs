// Oppress complaints inherited by metadata
#![allow(clippy::enum_variant_names)]

use subxt::{
	config::Config,
	ext::sp_runtime::traits::{IdentifyAccount, Verify},
	tx::PolkadotExtrinsicParams,
};

#[subxt::subxt(runtime_metadata_path = "artifacts/pere10900.scale")]
pub mod metadata {}

#[cfg(all(feature = "10801", not(feature = "default")))]
#[subxt::subxt(runtime_metadata_path = "artifacts/pere10801.scale")]
pub mod metadata {}

// re-export all the auto generated code
pub use metadata::{runtime_types::peregrine_runtime as kilt_runtime, *};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KiltConfig;
impl Config for KiltConfig {
	type Index = u64;
	type BlockNumber = u64;
	type Hash = subxt::ext::sp_core::H256;
	type Hashing = subxt::ext::sp_runtime::traits::BlakeTwo256;
	type AccountId = <<Self::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Address = subxt::ext::sp_runtime::MultiAddress<Self::AccountId, ()>;
	type Header = subxt::ext::sp_runtime::generic::Header<Self::BlockNumber, Self::Hashing>;
	type Signature = subxt::ext::sp_runtime::MultiSignature;
	type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}
