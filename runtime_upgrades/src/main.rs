mod cli;
mod extrinsics;
mod kilt;
mod migrations;
mod upgrade;
mod utility;

use clap::Parser;
use cli::Commands;
use sp_core::crypto::AccountId32;
use std::fs;
use subxt::OnlineClient;

use kilt::KiltConfig;
use sp_core::Pair;
use std::str::FromStr;
use subxt::ext::sp_runtime::traits::Zero;

const WASM_BLOB: &[u8] = include_bytes!("../artifacts/peregrine_10900rc0.wasm");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = cli::Args::parse();
	let api = OnlineClient::<KiltConfig>::from_url(args.websocket).await?;

	match &args.command {
		Commands::SpawnConnectedDids(spawn_cmd) => {
			#[cfg(feature = "pre-eth-migration")]
			{
				let (keypair, _) = sp_core::sr25519::Pair::from_string_with_seed(&spawn_cmd.mnemonic, None)
					.expect("Failed to create sr25519 keypair from provided mnemonic");

				<migrations::eth::EthMigration as migrations::eth::pre::PreEthMigration>::spawn_linked_dids(
					&migrations::eth::EthMigration(api),
					keypair,
					spawn_cmd.num_dids,
				)
				.await?;
			}
		}
		Commands::ExecuteRuntimeUpgrade(subargs) => {
			let wasm_blob: Vec<u8> = fs::read(&subargs.wasm_path)?;
			let (keypair, _) = sp_core::sr25519::Pair::from_string_with_seed(&subargs.mnemonic, None)
				.expect("Failed to create sr25519 keypair from provided mnemonic");
			crate::extrinsics::upgrade::execute_set_code(api.clone(), &wasm_blob, keypair).await?;
			// fs::write(args.last_upgrade_block_hash, hash.to_string());
		}
		Commands::RuntimeUpgradeSanityCheck(hash_input) => {
			let hash = sp_core::H256::from_str(&hash_input.hash).ok();
			upgrade::post_upgrade_sanity_checks(api.clone(), hash).await?
		}
		Commands::MigrateLinkableAccountIds(subargs) => {
			#[cfg(feature = "post-eth-migration")]
			{
				// let (keypair, _) =
				// sp_core::sr25519::Pair::from_string_with_seed(&mnemonic.
				// hex_seed, None) .expect("Failed to create sr25519 keypair
				// from provided mnemonic"); let account_ids = if
				// !args.account_ids.len().is_zero() {
				// 	utility::read_from_file_account_ids(args.account_ids)?
				// } else {
				// 	<migrations::eth::EthMigration as
				// migrations::eth::ConnectedAccountIds>::get(
				// 		&migrations::eth::EthMigration(api),
				// 	)
				// 	.await?
				// };
				// let migrating_accs =
				// crate::migrations::eth::get_connnected_account_ids(api.
				// clone()).await?;
				// crate::migrations::eth::post::migrate_account_ids_dynamically(api.clone(), migrating_accs).await?;
			}
		}
	}

	Ok(())
}
