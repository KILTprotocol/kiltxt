use async_trait::async_trait;
use sp_core::{crypto::AccountId32, ByteArray};
use subxt::OnlineClient;

use crate::kilt::KiltConfig;

#[derive(Debug, Clone)]
pub struct EthMigration(pub OnlineClient<KiltConfig>);

#[async_trait]

pub trait ConnectedAccountIds {
	async fn get(&self) -> anyhow::Result<Vec<AccountId32>>;
}

#[async_trait]
impl ConnectedAccountIds for EthMigration {
	async fn get(&self) -> anyhow::Result<Vec<AccountId32>> {
		let api = self.0.clone();
		let connected_dids_storage_root = subxt::dynamic::storage_root("DidLookup", "ConnectedDids");
		let mut iter = api.storage().iter(connected_dids_storage_root, 10, None).await?;
		let mut account_ids: Vec<AccountId32> = vec![];

		while let Some((key, did_details)) = iter.next().await? {
			let last_32_bytes: Vec<u8> = key.0.clone().into_iter().rev().take(32).rev().collect();
			let account_id = AccountId32::from_slice(&last_32_bytes[..])
				.or_else(|_| anyhow::bail!("Failed to convert byte array to AccountId"))?;
			account_ids.push(account_id);
		}
		println!("Number of connected accounts to DIDs: {:?}", account_ids.len());
		Ok(account_ids)
	}
}

#[cfg(feature = "pre-eth-migration")]
pub mod pre {
	use std::{char::MAX, ops::Deref};

	use async_trait::async_trait;
	use sp_core::Pair as PairT;
	use sp_keyring::sr25519::sr25519::Pair;
	use subxt::{ext::sp_runtime::traits::Zero, tx::PairSigner};

	use crate::extrinsics::did::{Did, DummyDid};

	use super::EthMigration;

	const MAX_DID_BATCH_SIZE: usize = 100;

	#[async_trait]
	pub trait PreEthMigration {
		/// Creates a specified amount of new dummy DIDs which are linked to the
		/// public account of the provided owner keypair.
		async fn spawn_linked_dids(&self, owner_pair: Pair, num_dids: u32) -> anyhow::Result<()>;
	}

	#[async_trait]
	impl PreEthMigration for EthMigration {
		async fn spawn_linked_dids(&self, owner_pair: Pair, num_dids: u32) -> anyhow::Result<()> {
			let api = self.0.clone();
			let current_block = api.blocks().at(None).await.unwrap().number();
			let signer = PairSigner::new(owner_pair.clone());

			println!(
				"Spawning {} new DIDs and linking to owner {:?}",
				num_dids,
				owner_pair.public()
			);
			// Build calls
			let mut calls = vec![];
			let mut batches = vec![];
			for _ in 0..num_dids {
				let proxy_keypair = crate::utility::create_proxy_keypair_sr25519()?;
				// Create dummy DID
				calls.push(DummyDid::create(owner_pair.public(), proxy_keypair.clone()));

				// Link dummy DID to owner's public address
				calls.push(DummyDid::link_account(
					owner_pair.public(),
					proxy_keypair.clone(),
					current_block,
				));

				if calls.len() >= MAX_DID_BATCH_SIZE {
					batches.push(crate::kilt::tx().utility().batch(calls));
					calls = vec![];
				}
			}
			batches.push(crate::kilt::tx().utility().batch(calls));

			// let batches: Vec<Vec<crate::kilt::kilt_runtime::Call>> = calls
			// 	.chunks(MAX_DID_BATCH_SIZE)
			// 	.map(|c| Box::new(c.into_iter().map(|call| *c).collect()).into_vec())
			// 	.collect();
			for tx in batches.into_iter() {
				let tx_progress = api.tx().sign_and_submit_then_watch_default(&tx, &signer).await?;
				let res = tx_progress.wait_for_in_block().await?;

				println!(
					"Executed batch call with hash {} in block {}",
					res.extrinsic_hash(),
					res.block_hash()
				);
			}

			Ok(())
		}
	}
}

// #[cfg(all(feature = "post-eth-migration", not(feature =
// "pre-eth-migration")))]
#[cfg(feature = "post-eth-migration")]
pub mod post {
	use crate::kilt::utility::calls::Batch;
	use codec::{Decode, Encode};
	use sp_core::crypto::AccountId32;
	use sp_keyring::AccountKeyring;
	use subxt::{
		dynamic::Value,
		ext::sp_runtime::traits::Zero,
		tx::{DynamicTxPayload, PairSigner},
		OnlineClient,
	};

	// Somehow this cannot be found via kilt::did_lookup::events::Migrated and thus
	// requires manual setup

	#[derive(Decode, Encode, Debug)]
	struct MigratedEvent {
		did_id: sp_core::H256,
		account_id: crate::kilt::runtime_types::pallet_did_lookup::linkable_account::LinkableAccountId,
	}
	impl subxt::events::StaticEvent for MigratedEvent {
		const PALLET: &'static str = "DidLookup";
		const EVENT: &'static str = "Migrated ";
	}

	const MAX_BATCH_SIZE: usize = 1;

	// TODO: Rather expose functions via Interface
	pub async fn migrate_account_ids(
		api: OnlineClient<crate::kilt::KiltConfig>,
		account_ids: Vec<AccountId32>,
	) -> anyhow::Result<()> {
		// TODO: Should be provided via input
		let alice_pair = AccountKeyring::Alice.pair();
		let alice = PairSigner::new(alice_pair.clone());
		let mut batches = vec![];
		let mut calls = vec![];
		let ids = account_ids.into_iter().rev().take(300).clone();
		println!("Initiating ETH Migration for {} accounts", ids.len());

		for account_id in ids.clone() {
			let tx = crate::kilt::RuntimeCall::DidLookup(
				crate::kilt::runtime_types::pallet_did_lookup::pallet::Call::migrate_account_id {
					account_id: account_id.clone(),
				},
			);

			// let tx = subxt::dynamic::tx(
			// 	"DidLookup",
			// 	"migrate_account_id",
			// 	vec![Value::unnamed_variant("Id", [Value::from_bytes(&account_id)])],
			// );
			calls.push(tx);

			if calls.len() == MAX_BATCH_SIZE || ids.clone().last() == Some(account_id) {
				println!("Preparing next batch of DID account migration {}", calls.len());
				let tx = crate::kilt::tx().utility().batch(calls);
				calls = vec![];

				let res = api.tx().sign_and_submit_then_watch_default(&tx, &alice).await?;
				batches.push(res);
			}
		}

		// submit batch calls
		for handle in batches {
			let res = handle.wait_for_in_block().await.unwrap();
			println!("Done with migration batch in extrinsic {}", res.extrinsic_hash());
		}

		Ok(())
	}

	// TODO: Rather expose functions via Interface
	pub async fn migrate_account_ids_dynamically(
		api: OnlineClient<crate::kilt::KiltConfig>,
		account_ids: Vec<AccountId32>,
	) -> anyhow::Result<Vec<AccountId32>> {
		// TODO: Should be provided via input
		let alice_pair = AccountKeyring::Alice.pair();
		let alice = PairSigner::new(alice_pair.clone());
		let mut batches = vec![];
		let mut calls = vec![];
		let ids = account_ids.into_iter().rev().take(5).clone();
		let mut failed_migrations = vec![vec![]];
		let mut i = 0usize;

		println!("Initiating ETH Migration for {} accounts", ids.len());
		for account_id in ids.clone() {
			i += 1;
			let tx = subxt::dynamic::tx("DidLookup", "migrate_account_id", vec![Value::from_bytes(&account_id)]);
			calls.push(tx);
			failed_migrations[i % MAX_BATCH_SIZE].push(account_id);

			if i == MAX_BATCH_SIZE.min(ids.len() - 1) {
				println!("Preparing next batch of DID account migration {}", calls.len());
				let call_values: Vec<Value> = calls
					.into_iter()
					.map(|call: DynamicTxPayload| call.into_value())
					.collect();
				let tx = subxt::dynamic::tx("Utility", "batch", vec![("calls", call_values)]);
				calls = vec![];

				let res = api.tx().sign_and_submit_then_watch_default(&tx, &alice).await?;
				batches.push(res);
			}
		}

		// submit batch calls
		i = 0usize;
		for handle in batches {
			let res = handle.wait_for_in_block().await.unwrap();
			let events = res.fetch_events().await?;
			let success_events = events.find::<MigratedEvent>();
			for success in success_events {
				println!("Success {:?}", success?);
				println!("Migrated account ids {:?}", failed_migrations[i]);
				failed_migrations[i].clear();
			}
			let failure_events = events.find::<crate::kilt::utility::events::BatchInterrupted>();
			for failure in failure_events {
				println!("Batch Interrupted {:?}", failure?);
				println!("Failed to migrate account ids {:?}", failed_migrations[i]);
			}
			println!("Done with migration batch in extrinsic {}", res.extrinsic_hash());
			i += 1;
		}

		let x: Vec<AccountId32> = failed_migrations.into_iter().flatten().collect();

		Ok(x)
	}
}
