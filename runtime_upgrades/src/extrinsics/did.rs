use codec::Encode;
use sp_keyring::sr25519::sr25519::Public;
use subxt::ext::{sp_core::crypto::Pair, sp_runtime::AccountId32};

use crate::kilt::runtime_types::did::did_details::{DidCreationDetails, DidSignature};

const PAYLOAD_BYTES_WRAPPER_PREFIX: &[u8; 7] = b"<Bytes>";
const PAYLOAD_BYTES_WRAPPER_POSTFIX: &[u8; 8] = b"</Bytes>";
const DID_EXPIRATION: u64 = 5_000_000;

// TODO: Check how to simplify
fn get_wrapped_payload(payload: &[u8]) -> Vec<u8> {
	PAYLOAD_BYTES_WRAPPER_PREFIX
		.iter()
		.chain(payload.iter())
		.chain(PAYLOAD_BYTES_WRAPPER_POSTFIX.iter())
		.copied()
		.collect()
}

/// Creates a dummy DID defaulting to sr25519 signature
pub(crate) fn dummy_create_did(
	public_pair: Public,
	did_key: sp_keyring::sr25519::sr25519::Pair,
) -> crate::kilt::RuntimeCall {
	let creation_details = DidCreationDetails {
		did: did_key.public().into(),
		submitter: public_pair.into(),
		new_key_agreement_keys: crate::kilt::runtime_types::sp_runtime::bounded::bounded_btree_set::BoundedBTreeSet(
			vec![],
		),
		new_attestation_key: None,
		new_delegation_key: None,
		new_service_details: vec![],
	};

	let signature = DidSignature::Sr25519(crate::kilt::runtime_types::sp_core::sr25519::Signature(
		did_key.sign(&creation_details.encode()).0,
	));

	crate::kilt::RuntimeCall::Did(crate::kilt::runtime_types::did::pallet::Call::create {
		details: Box::new(creation_details),
		signature,
	})
}

#[cfg(feature = "pre-eth-migration")]
/// Creates a dummy DID defaulting to sr25519 signature
pub(crate) fn dummy_link_account_with_did(
	public_key: Public,
	did_keypair: sp_keyring::sr25519::sr25519::Pair,
	current_block: u64,
) -> crate::kilt::RuntimeCall {
	let addr = AccountId32::from(public_key);

	let payload: Vec<u8> = get_wrapped_payload(&(&addr, DID_EXPIRATION).encode());
	let proof = crate::kilt::runtime_types::sp_runtime::MultiSignature::Sr25519(
		crate::kilt::runtime_types::sp_core::sr25519::Signature(did_keypair.sign(&payload).0),
	);
	let associate_account_call = crate::kilt::RuntimeCall::DidLookup(
		crate::kilt::runtime_types::pallet_did_lookup::pallet::Call::associate_account {
			account: did_keypair.public().into(),
			expiration: DID_EXPIRATION,
			proof,
		},
	);
	let did_call = crate::kilt::runtime_types::did::did_details::DidAuthorizedCallOperation {
		did: addr,
		tx_counter: 1,
		call: associate_account_call,
		block_number: current_block,
		submitter: public_key.into(),
	};
	let signature = DidSignature::Sr25519(crate::kilt::runtime_types::sp_core::sr25519::Signature(
		did_keypair.sign(&did_call.encode()).0,
	));

	crate::kilt::RuntimeCall::Did(crate::kilt::runtime_types::did::pallet::Call::submit_did_call {
		did_call: Box::new(did_call),
		signature,
	})
}
