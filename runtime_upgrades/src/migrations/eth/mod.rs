use async_trait::async_trait;
use sp_core::{crypto::AccountId32, ByteArray};
use subxt::OnlineClient;

#[derive(Debug, Clone)]
pub struct EthMigration(pub OnlineClient<crate::kilt::KiltConfig>);

#[async_trait]

pub trait ConnectedAccountIds {
	/// Query keys of `ConnectedDids` from chain which reflects all account ids
	/// which need to be migrated to linkable ones.
	async fn get_from_chain(&self) -> anyhow::Result<Vec<AccountId32>>;
}

#[async_trait]
impl ConnectedAccountIds for EthMigration {
	async fn get_from_chain(&self) -> anyhow::Result<Vec<AccountId32>> {
		let api = self.0.clone();
		let connected_dids_storage_root = subxt::dynamic::storage_root("DidLookup", "ConnectedDids");
		let mut iter = api.storage().iter(connected_dids_storage_root, 10, None).await?;
		let mut account_ids: Vec<AccountId32> = vec![];

		while let Some((key, _)) = iter.next().await? {
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
	use async_trait::async_trait;
	use sp_core::Pair as PairT;
	use sp_keyring::sr25519::sr25519::Pair;
	use subxt::tx::PairSigner;

	use crate::{
		extrinsics::did::{Did, DummyDid},
		kilt, utility,
	};

	use super::EthMigration;

	const MAX_DID_BATCH_SIZE: usize = 200;

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
				owner_pair.public().to_string()
			);

			// Build batch calls
			println!("Preparing calls for DID creation and account association");
			let mut calls = vec![];
			let mut batches = vec![];
			for _ in 0..num_dids {
				let proxy_keypair = utility::create_proxy_keypair_sr25519()?;
				// Create dummy DID
				calls.push(DummyDid::create(owner_pair.public(), proxy_keypair.clone()));

				// Link dummy DID to owner's public address
				calls.push(DummyDid::link_account(
					owner_pair.public(),
					proxy_keypair.clone(),
					current_block,
				));

				if calls.len() >= MAX_DID_BATCH_SIZE {
					batches.push(kilt::tx().utility().batch(calls));
					calls = vec![];
				}
			}
			batches.push(kilt::tx().utility().batch(calls));

			// Execute batch calls
			println!("Starting to execute calls in batches of size <= {}", MAX_DID_BATCH_SIZE);
			for tx in batches.into_iter() {
				let tx_progress = api.tx().sign_and_submit_then_watch_default(&tx, &signer).await?;
				println!("\t Signed");
				let res = tx_progress.wait_for_in_block().await?;

				println!(
					"Executed batch call with hash {} in block {}",
					res.extrinsic_hash(),
					res.block_hash()
				);
				let events = res.fetch_events().await?;
				if events
					.find_first::<kilt::utility::events::BatchInterrupted>()?
					.is_some()
				{
					println!("\t Sadly, the batch did not go through but was interrupted");
				}
			}
			println!(
				"Done! All {} DIDs have been created and linked to {}",
				num_dids,
				owner_pair.public().to_string()
			);

			Ok(())
		}
	}
}

#[cfg(not(feature = "pre-eth-migration"))]
pub mod post {
	use super::{ConnectedAccountIds, EthMigration};
	use crate::{
		kilt,
		kilt::{
			kilt::did_lookup::events::{Migrated, MigrationCompleted},
			KiltConfig,
		},
	};
	use async_trait::async_trait;
	use sp_core::{crypto::AccountId32, H256};
	use sp_keyring::sr25519::sr25519::Pair;
	use subxt::{blocks::ExtrinsicEvents, dynamic::Value, events::Events, tx::PairSigner};

	const MAX_BATCH_SIZE: usize = 89;

	#[async_trait]
	pub trait ExecuteAccountMigration {
		/// Execute the extrinsic to finalize migration. On success, sets flag
		/// to false and re-enables all DidPallet calls.
		async fn finalize_migration(&self, signer_pair: Pair) -> anyhow::Result<()>;

		/// Executes the on-chain migration of AccountId keytypes to
		/// LinkableAccountId by calling the appropriate migration extrinsic on
		/// the chain.
		async fn migrate_account_ids(&self, signer_pair: Pair, migrated_id_path: String) -> anyhow::Result<()>;

		/// Query account ids from chain and remove already migrated ones based
		/// on result of [`update_migrated`].
		async fn get_migrated_ids(&self, file_path: String) -> anyhow::Result<Vec<AccountId32>>;

		/// Update local "database" of migrated account ids.
		fn update_migrated_db(newly_migrated: Vec<AccountId32>, file_path: String) -> anyhow::Result<()>;

		/// Handler for the [`Migrated`] extrinsic event. Iterates extrinsic
		/// events and checks for migrated account ids.
		fn success_event_handler(events: &Events<KiltConfig>) -> anyhow::Result<Vec<AccountId32>>;

		/// Traverses blocks since last runtime upgrade and filter for migrated
		/// account ids.
		///
		/// NOTE: For local chains, this only works for the last 258 blocks
		/// since collators are not a full node.
		async fn traverse_migration_events(&self, upgrade_block: H256) -> anyhow::Result<Vec<AccountId32>>;
	}

	#[async_trait]
	impl ExecuteAccountMigration for EthMigration {
		async fn get_migrated_ids(&self, file_path: String) -> anyhow::Result<Vec<AccountId32>> {
			// Read account ids from file
			let mut migrated = crate::utility::fs_read_migrated_ids(file_path.clone())?;
			migrated.dedup();
			// Read account ids from chain
			let all_ids = EthMigration::get_from_chain(&self).await?;
			println!(
				"Number of accounts which were already migrated: {} vs. total {}",
				migrated.len(),
				all_ids.len()
			);
			let unmigrated: Vec<AccountId32> = all_ids
				.clone()
				.into_iter()
				.filter(|id| !migrated.iter().any(|r| r == id))
				.collect();
			println!("Number of unmigrated accouts: {}", unmigrated.len());
			Ok(unmigrated)
		}

		fn update_migrated_db(newly_migrated: Vec<AccountId32>, file_path: String) -> anyhow::Result<()> {
			crate::utility::fs_append_migrated_ids(file_path.clone(), newly_migrated)
		}

		async fn migrate_account_ids(&self, signer_pair: Pair, migrated_id_path: String) -> anyhow::Result<()> {
			let unmigrated = Self::get_migrated_ids(&self, migrated_id_path.clone()).await?;
			let account_chunks: Vec<Vec<AccountId32>> =
				unmigrated.chunks(MAX_BATCH_SIZE).map(|chunk| chunk.into()).collect();

			// Get migrated events
			println!("Preparing calls for account ID migration");
			let signer = PairSigner::new(signer_pair.clone());
			for chunk in account_chunks {
				let call_values: Vec<Value> = chunk
					.iter()
					.map(|account_id| {
						subxt::dynamic::tx("DidLookup", "migrate_account_id", vec![Value::from_bytes(account_id)])
					})
					.map(|call| call.into_value())
					.collect();
				println!("\t Created batch of size {}", call_values.len());
				let tx = subxt::dynamic::tx("Utility", "batch", vec![("calls", call_values)]);
				let signed_tx = self.0.tx().sign_and_submit_then_watch_default(&tx, &signer).await?;
				println!("\t Signed. Waiting for in block...");

				let res = signed_tx.wait_for_in_block().await.unwrap();
				let events: ExtrinsicEvents<KiltConfig> = res.fetch_events().await?;
				let events2: &Events<KiltConfig> = events.all_events_in_block();
				println!("Done with migration batch in extrinsic {}", res.extrinsic_hash());

				let migrated_ids = Self::success_event_handler(events2)?;
				Self::update_migrated_db(migrated_ids, migrated_id_path.clone())?;

				// println!("All events {:?}", events.all_events_in_block());
				if events
					.find_first::<kilt::utility::events::BatchInterrupted>()?
					.is_some()
				{
					println!("\t Sadly, the batch did not go through but was interrupted. Some account ids of the batch must already be migrated");
				}
			}
			Ok(())
		}

		async fn finalize_migration(&self, signer_pair: Pair) -> anyhow::Result<()> {
			let tx = subxt::dynamic::tx("DidLookup", "try_finalize_migration", Vec::<Value>::new());
			let signed_tx = self
				.0
				.tx()
				.sign_and_submit_then_watch_default(&tx, &PairSigner::new(signer_pair.clone()))
				.await?;
			let res = signed_tx.wait_for_in_block().await.unwrap();
			let events = res.fetch_events().await?;
			println!(
				"Done with migration finalization try in extrinsic {}",
				res.extrinsic_hash()
			);

			if events.find_first::<MigrationCompleted>()?.is_some() {
				println!("\t Migration finalized!");
				Ok(())
			} else {
				anyhow::bail!("Migration could not be finalized. Some account ids still need to be migrated.");
			}
		}

		fn success_event_handler(events: &Events<KiltConfig>) -> anyhow::Result<Vec<AccountId32>> {
			let account_ids = events
				.find::<Migrated>()
				.filter_map(|res| {
					res.map(|success| {
						if let kilt::runtime_types::pallet_did_lookup::linkable_account::LinkableAccountId::AccountId32(
						account_id,
					) = success.account_id
					{
						Some(account_id)
					} else {
						None
					}
					})
					.unwrap_or(None)
				})
				.collect();
			Ok(account_ids)
		}

		async fn traverse_migration_events(&self, upgrade_block: H256) -> anyhow::Result<Vec<AccountId32>> {
			crate::migrations::filter_blocks_for_event(self.0.clone(), Some(upgrade_block), Self::success_event_handler)
				.await
		}
	}
}
