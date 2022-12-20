mod cli;
mod extrinsics;
mod kilt;
mod migrations;
mod upgrade;
mod utility;

use clap::Parser;
use cli::Commands;
use std::fs;
use subxt::OnlineClient;

use kilt::KiltConfig;
use sp_core::Pair;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = cli::Args::parse();
	let api = OnlineClient::<KiltConfig>::from_url(args.websocket).await?;

	match &args.command {
		#[cfg(all(feature = "10801", not(feature = "default")))]
		Commands::SpawnConnectedDids(spawn_cmd) => {
			let (keypair, _) = sp_core::sr25519::Pair::from_string_with_seed(&spawn_cmd.seed, None)
				.expect("Failed to create sr25519 keypair from provided mnemonic");

			<migrations::eth::EthMigration as migrations::eth::pre::PreEthMigration>::spawn_linked_dids(
				&migrations::eth::EthMigration(api),
				keypair,
				spawn_cmd.num_dids,
			)
			.await?;
		}
		Commands::ExecuteRuntimeUpgrade(upgrade_cmd) => {
			let wasm_blob: Vec<u8> = fs::read(&upgrade_cmd.wasm_path)?;
			let (keypair, _) = sp_core::sr25519::Pair::from_string_with_seed(&upgrade_cmd.seed, None)
				.expect("Failed to create sr25519 keypair from provided mnemonic");
			crate::extrinsics::upgrade::execute_set_code(api.clone(), &wasm_blob, keypair).await?;

			// TODO: Feat: Listen to subsequent blocks for
			// parachain_system::events::ValidationFunctionApplied which should
			// happen a couple of blocks later
		}
		Commands::RuntimeUpgradeSanityCheck(sanity_cmd) => {
			let pre_upgrade = sp_core::H256::from_str(&sanity_cmd.hash).ok();
			upgrade::post_upgrade_sanity_checks(api.clone(), None, pre_upgrade).await?
		}
		Commands::MigrateLinkableAccountIds(migrate_cmd) => {
			{
				let (keypair, _) = sp_core::sr25519::Pair::from_string_with_seed(&migrate_cmd.seed, None).expect(
					"Failed to create sr25519 keypair
				from provided mnemonic",
				);

				// Only required in case [`migrate_account_ids`] finished and did not store all
				// migrated ids in local database.
				if let Some(stop_hash) = &migrate_cmd.traverse_blocks_stop_hash {
					let newly_migrated = <migrations::eth::EthMigration as
					migrations::eth::post::ExecuteAccountMigration>::traverse_migration_events(
						&migrations::eth::EthMigration(api.clone()),
						sp_core::H256::from_str(stop_hash).expect("Failed to derive upgrade block") )
					.await?;
					<migrations::eth::EthMigration as migrations::eth::post::ExecuteAccountMigration>::update_migrated_db(
						newly_migrated,
						migrate_cmd.migrated_db.clone(),
					)?;
				}

				<migrations::eth::EthMigration as migrations::eth::post::ExecuteAccountMigration>::migrate_account_ids(
					&migrations::eth::EthMigration(api.clone()),
					keypair.clone(),
					migrate_cmd.migrated_db.clone(),
				)
				.await?;

				<migrations::eth::EthMigration as migrations::eth::post::ExecuteAccountMigration>::finalize_migration(
					&migrations::eth::EthMigration(api),
					keypair,
				)
				.await?;
			}
		}
	}

	Ok(())
}
