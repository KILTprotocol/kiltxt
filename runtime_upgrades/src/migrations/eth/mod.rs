use sp_core::{crypto::AccountId32, ByteArray};
use subxt::{dynamic::Value, OnlineClient};

pub async fn get_connnected_account_ids(
	api: OnlineClient<crate::kilt::KiltConfig>,
) -> anyhow::Result<Vec<AccountId32>> {
	let connected_dids_storage_root = subxt::dynamic::storage_root("DidLookup", "ConnectedDids");
	let mut iter = api.storage().iter(connected_dids_storage_root, 10, None).await?;
	let mut counter = 0u32;
	let mut account_ids: Vec<AccountId32> = vec![];
	// TODO: Change to while
	if let Some((key, did_details)) = iter.next().await? {
		println!(
			"{}th key: {} \n\t value: {}",
			counter,
			hex::encode(key.clone()),
			did_details.to_value()?
		);
		counter += 1;
		let last_32_bytes: Vec<u8> = key.0.clone().into_iter().rev().take(32).rev().collect();
		let account_id = AccountId32::from_slice(&last_32_bytes[..])
			.or_else(|_| anyhow::bail!("Failed to convert byte array to AccountId"))?;
		account_ids.push(account_id);

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

	Ok(account_ids)
}

#[cfg(feature = "pre-eth-migration")]
pub mod pre {

	use bip39::{Language, Mnemonic, MnemonicType};
	use sp_keyring::AccountKeyring;
	use subxt::{
		ext::{
			sp_core::Pair,
			sp_runtime::{app_crypto::sr25519, SaturatedConversion},
		},
		tx::PairSigner,
		OnlineClient,
	};

	use crate::extrinsics;

	const MAX_DID_BATCH_SIZE: usize = 100;

	fn create_tmp_keypair() -> anyhow::Result<sp_keyring::sr25519::sr25519::Pair> {
		// create new DID saving seed in the seeds vector
		let m_type = MnemonicType::for_word_count(12)?;
		let mnemonic = Mnemonic::new(m_type, Language::English);
		let m: String = mnemonic.phrase().into();
		sr25519::Pair::from_string_with_seed(&m, None)
			.map(|(pair, _)| pair)
			.or_else(|_| anyhow::bail!("Failed to create sr25519 keypair from random mnemonic {}", mnemonic))
	}

	// TODO: Rather expose functions via Interface
	pub async fn spawn_linked_dids(api: OnlineClient<crate::kilt::KiltConfig>, num_dids: u32) -> anyhow::Result<()> {
		println!("Spawning {} new DIDs and linking to Alice", num_dids);

		// TODO: Should be provided via input
		let alice_pair = AccountKeyring::Alice.pair();
		let alice = PairSigner::new(alice_pair.clone());
		let current_block = api.blocks().at(None).await.unwrap().number();

		// build batch calls
		let mut calls = vec![];
		let mut handles = vec![];
		for i in 0..num_dids {
			let keypair = create_tmp_keypair()?;

			// create batch tx whenever max batch size is reached or end of loop
			if calls.len() == MAX_DID_BATCH_SIZE || i == num_dids - 1 {
				println!(
					"[#{}/{}] Preparing DID create + link account batch call of size {}",
					i / MAX_DID_BATCH_SIZE.saturated_into::<u32>() + 1,
					num_dids / MAX_DID_BATCH_SIZE.saturated_into::<u32>() + 1,
					calls.len()
				);

				let tx = crate::kilt::tx().utility().batch(calls);
				calls = vec![];

				let res = api.tx().sign_and_submit_then_watch_default(&tx, &alice).await?;

				handles.push(res);
			}

			// create dummy DID
			calls.push(extrinsics::did::dummy_create_did(alice_pair.public(), keypair.clone()));

			// link dummy DID to Alice's address
			calls.push(extrinsics::did::dummy_link_account_with_did(
				alice_pair.public(),
				keypair.clone(),
				current_block,
			));
		}

		// submit batch calls
		for handle in handles {
			let res = handle.wait_for_in_block().await.unwrap();
			println!("Done with batch in extrinsic {}", res.extrinsic_hash());
		}

		Ok(())
	}
}

#[cfg(feature = "post-eth-migration")]
pub mod post {
	use crate::kilt::utility::calls::Batch;
	use sp_core::crypto::AccountId32;
	use sp_keyring::AccountKeyring;
	use subxt::{
		dynamic::Value,
		ext::sp_runtime::traits::Zero,
		tx::{DynamicTxPayload, PairSigner},
		OnlineClient,
	};

	const MAX_BATCH_SIZE: usize = 200;

	// TODO: Rather expose functions via Interface
	pub async fn migrate_account_ids(
		api: OnlineClient<crate::kilt::KiltConfig>,
		account_ids: Vec<AccountId32>,
	) -> anyhow::Result<()> {
		// TODO: Should be provided via input
		let alice_pair = AccountKeyring::Alice.pair();
		let alice = PairSigner::new(alice_pair.clone());
		let mut handles = vec![];
		let mut calls = vec![];
		println!("Initiating ETH Migration for {} accounts", account_ids.len());

		for account_id in account_ids.clone() {
			if calls.len() == MAX_BATCH_SIZE || account_ids.last() == Some(&account_id) {
				println!("Preparing next batch of DID account migration {}", calls.len());
				let call_values: Vec<Value> = calls
					.into_iter()
					.map(|call: DynamicTxPayload| call.into_value())
					.collect();
				let tx = subxt::dynamic::tx("Utility", "batch", vec![("call", call_values)]);
				calls = vec![];

				let res = api.tx().sign_and_submit_then_watch_default(&tx, &alice).await?;
				handles.push(res);
			}
			let tx = subxt::dynamic::tx(
				"DidLookup",
				"migrate_account_id",
				vec![Value::unnamed_variant("Id", [Value::from_bytes(&account_id)])],
			);
			calls.push(tx)
		}

		Ok(())
	}
}
