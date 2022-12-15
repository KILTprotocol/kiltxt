use clap::{Parser, Subcommand};

use crate::migrations::eth::EthMigration;

const LOCAL_WS: &str = "ws://localhost:40047";
const LAST_UPGRADE_BLOCK_HASH: &str = "artifacts/last_upgrade_block.txt";
// TODO: Check clap attribute for helpers etc
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	/// Websocket endpoint of the source chain
	#[clap(short, long, value_parser, default_value_t = LOCAL_WS.to_string())]
	pub websocket: String,

	/// Executable action
	#[command(subcommand)]
	pub command: Commands,
	// /// Mnemomic hex for signing
	// #[arg(short, long, required = false, requires_ifs = [("SpawnConnectedDids", "call"), ("ExecuteRuntimeUpgrade",
	// "call"), ("MigrateDidLinkableAccountIds", "call")])] pub mnemonic: String,

	// /// WASM path required for runtime upgrades
	// #[arg(long, required = false, requires_ifs = [("ExecuteRuntimeUpgrade", "call")])]
	// pub wasm: String,
	// /// Block in which last upgrade happened
	// #[arg(short = 'h', long, required = false, requires_ifs = [("RuntimeUpgradeSanityCheck", "call")])]
	// pub last_upgrade_block_hash: String,

	// /// Path to migrating account ids
	// #[arg(short, long, required = false, requires_ifs = [("MigrateDidLinkableAccountIds", "call")])]
	// pub account_ids: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	SpawnConnectedDids(SpawnCmd),
	ExecuteRuntimeUpgrade(UpgradeCmd),
	RuntimeUpgradeSanityCheck(SanityCheckCmd),
	MigrateLinkableAccountIds(EthMigrationCmd),
}

#[derive(clap::Args, Debug)]
pub struct SanityCheckCmd {
	#[arg(
		short,
		long,
		help = "Please provide the blockhash of the last runtime upgrade in hex format, 
				e.g. 0x... \n\n You can query it via $ subalfred get runtime-upgrade-block $spec_version --uri $websocket"
	)]
	pub hash: String,
}

#[derive(clap::Args, Debug)]
pub struct EthMigrationCmd {
	#[arg(
		short,
		long,
		help = "Please provide the path of the file storing already migrated account ids"
	)]
	pub account_ids: String,
	#[arg(
		short,
		long,
		help = "Please provide the migration signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
	)]
	pub mnemonic: String,
}

#[derive(clap::Args, Debug)]
pub struct UpgradeCmd {
	#[arg(
		short,
		long,
		help = "Please provide the path of the WASM file required for the runtime upgrade"
	)]
	pub wasm_path: String,
	#[arg(
		short,
		long,
		help = "Please provide the SUDO signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
	)]
	pub mnemonic: String,
}

#[derive(clap::Args, Debug)]
pub struct SpawnCmd {
	#[arg(
		short,
		long,
		help = "Please provide the signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
	)]
	pub mnemonic: String,
	#[arg(
		short,
		long,
		help = "Please provide the number of dids which should be spawned and connected to the public address of the mnemonic."
	)]
	pub num_dids: u32,
}
