mod extrinsics;
mod kilt;
mod migrations;
mod upgrade;

use codec::Decode;
use subxt::{
	ext::{scale_value::ValueDef, sp_core::crypto::Ss58Codec},
	OnlineClient,
};

use kilt::KiltConfig;
use sp_core::{
	crypto::AccountId32,
	sr25519::{self, Public},
	twox_128, Pair,
};
use sp_keyring::Sr25519Keyring;
use subxt::dynamic::Value;

const WASM_BLOB: &[u8] = include_bytes!("../artifacts/peregrine_10900rc0.wasm");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let api = OnlineClient::<KiltConfig>::from_url("ws://localhost:40047").await?;

	let storage_address = subxt::dynamic::storage_root("DidLookup", "ConnectedDids");
	let mut iter = api.storage().iter(storage_address, 1, None).await?;
	let mut counter = 0u32;
	if let Some((key, did_details)) = iter.next().await? {
		println!(
			"{}th key: {} \n\t value: {:?}",
			counter,
			hex::encode(key.clone()),
			did_details.to_value()?.value
		);
		counter += 1;
		let last_32_bytes: Vec<u8> = key.0.clone().into_iter().rev().take(32).rev().collect();
		let public: Public = Public::try_from(&last_32_bytes[..]).unwrap();
		// println!("Account Id {:?}", public.to_ss58check());
		// println!("Hex 0x{:?}", hex::encode(public));

		// let tx = subxt::dynamic::tx(
		//     "Balances",
		//     "transfer",
		//     vec![
		//         Value::unnamed_variant("Id", [Value::from_bytes(&public)]),
		//         Value::u128(123_456_789_012),
		//     ],
		// );
		// let signer =
		// subxt::tx::PairSigner::new(sp_keyring::AccountKeyring::Bob.pair());
		// let hash = api.tx().sign_and_submit_default(&tx, &signer).await?;
		// println!("Balance transfer extrinsic submitted: {}", hash);
	}

	#[cfg(feature = "pre-eth-migration")]
	migrations::eth::pre::spawn_linked_dids(api.clone(), 0).await?;

	// Upgrade
	// TODO: Only execute if specified in args
	let mut post_upgrade_hash: Option<sp_core::H256> = None;
	#[cfg(feature = "upgrade-set-code")]
	{
		let sudo_pair = sp_keyring::AccountKeyring::Alice.pair();
		post_upgrade_hash = crate::extrinsics::upgrade::execute_set_code(api.clone(), WASM_BLOB, sudo_pair).await?;
	}

	// Post upgrade sanity checks
	upgrade::post_upgrade_sanity_checks(api.clone(), post_upgrade_hash).await?;

	#[cfg(feature = "post-eth-migration")]
	{
		let migrating_accs = crate::migrations::eth::get_connnected_account_ids(api.clone()).await?;
		crate::migrations::eth::post::migrate_account_ids(api.clone(), migrating_accs).await?;
	}

	Ok(())
}
