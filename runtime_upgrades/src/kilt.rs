use subxt::{
	config::Config,
	ext::sp_runtime::traits::{IdentifyAccount, Verify},
	tx::PolkadotExtrinsicParams,
};

#[cfg(feature = "post-eth-migration")]
#[subxt::subxt(runtime_metadata_path = "artifacts/local-pere10900rc0.scale")]
pub mod kilt {}

#[cfg(not(feature = "post-eth-migration"))]
#[subxt::subxt(runtime_metadata_path = "artifacts/local-pere.scale")]
pub mod kilt {}

// re-export all the auto generated code
pub use kilt::{
	runtime_types::{did::pallet as did, pallet_did_lookup::pallet as did_lookup, peregrine_runtime as kilt_runtime},
	*,
};

pub type RuntimeCall = kilt_runtime::Call;
pub type RuntimeEvent = kilt_runtime::Event;

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
