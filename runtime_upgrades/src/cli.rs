use clap::{Parser, Subcommand};

const LOCAL_WS: &str = "ws://localhost:40047";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
	/// Websocket endpoint of the source chain
	#[clap(short, long, value_parser, default_value_t = LOCAL_WS.to_string())]
	pub websocket: String,

	/// Executable action
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
	#[cfg(all(feature = "10801", not(feature = "default")))]
	SpawnConnectedDids(SpawnCmd),
	ExecuteRuntimeUpgrade(UpgradeCmd),
	RuntimeUpgradeSanityCheck(SanityCheckCmd),
	MigrateLinkableAccountIds(EthMigrationCmd),
}

#[cfg(all(feature = "10801", not(feature = "default")))]
#[derive(clap::Args, Debug)]
pub struct SpawnCmd {
	#[arg(
		short,
		long,
		default_value_t = String::from("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
		help = "Please provide the signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
	)]
	pub seed: String,
	#[arg(
		short,
		long,
		default_value_t = 1u32,
		help = "Please provide the number of dids which should be spawned and connected to the public address of the mnemonic."
	)]
	pub num_dids: u32,
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
		default_value_t = String::from("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
		help = "Please provide the SUDO signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a",
	)]
	pub seed: String,
}

#[derive(clap::Args, Debug)]
pub struct SanityCheckCmd {
	#[arg(
		long,
		help = "Please provide the blockhash of the last runtime upgrade in hex format, 
				e.g. 0x... \n\n For full nodes, you can query it via $ subalfred get runtime-upgrade-block $spec_version --uri $websocket"
	)]
	pub hash: String,
}

#[derive(clap::Args, Debug)]
pub struct EthMigrationCmd {
	#[arg(
		short,
		long,
		default_value_t = String::from("artifacts/account_ids.txt"),
		help = "Please provide the path of the file storing already migrated account ids."
	)]
	pub migrated_db: String,
	#[arg(
		short,
		long,
		default_value_t = String::from("0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"),
		help = "Please provide the migration signer mnemonic in hex format, e.g. Alice is 0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a"
	)]
	pub seed: String,
	#[arg(
		short,
		long,
		default_value = None,
		help = "In case you want to traverse back blocks from current head to filter for `Migrated` event, please provide the hash of the runtime upgrade.
				\n\n For full nodes, you can query it via $ subalfred get runtime-upgrade-block $spec_version --uri $websocket"
	)]
	pub traverse_blocks_stop_hash: Option<String>,
}
