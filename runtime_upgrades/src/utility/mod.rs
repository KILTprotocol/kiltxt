use sp_core::crypto::AccountId32;
use std::{
	io::{BufRead, Write},
	str::FromStr,
};

#[cfg(all(feature = "10801", not(feature = "default")))]
use subxt::ext::{sp_core::Pair, sp_runtime::app_crypto::sr25519};

#[cfg(all(feature = "10801", not(feature = "default")))]
// Creates a temporary sr25519 keypair based on a random mnemonic which is not
// expected to be required in the future
pub fn create_proxy_keypair_sr25519() -> anyhow::Result<sp_keyring::sr25519::sr25519::Pair> {
	let m_type = bip39::MnemonicType::for_word_count(12)?;
	let mnemonic = bip39::Mnemonic::new(m_type, bip39::Language::English);
	let m: String = mnemonic.phrase().into();
	sr25519::Pair::from_string_with_seed(&m, None)
		.map(|(pair, _)| pair)
		.or_else(|_| anyhow::bail!("Failed to create sr25519 keypair from random mnemonic {}", mnemonic))
}

/// Read a vector of account ids from the given file. Expects values to be
/// separated by new lines.
pub fn fs_read_migrated_ids(path: String) -> anyhow::Result<Vec<AccountId32>> {
	let file = std::fs::File::open(path).expect("Failed to open list of account ids to migrate");
	let mut account_ids = vec![];
	let reader = std::io::BufReader::new(file);
	for line in reader.lines() {
		let content = line?;
		let id = AccountId32::from_str(&content)
			.or_else(|_| anyhow::bail!(format!("Failed to write line {:?} to file", &content)))?;
		account_ids.push(id);
	}
	println!("Read {:?} account_ids from input file", account_ids.len());
	Ok(account_ids)
}

/// Appends a vector of account ids to the specified file. Writes each value to
/// a separate line.
pub fn fs_append_migrated_ids(path: String, account_ids: Vec<AccountId32>) -> anyhow::Result<()> {
	let mut file = std::fs::OpenOptions::new().write(true).append(true).open(path)?;
	for account_id in account_ids {
		writeln!(file, "{}", account_id)?;
	}
	Ok(())
}

// pub trait EventHandler {
// 	fn exists<Ev, F, R>(events: ExtrinsicEvents<KiltConfig>, handler: F) ->
// anyhow::Result<Option<R>> 	where
// 		Ev: StaticEvent,
// 		F: Fn(Ev) -> R,
// 		dyn Fn(Ev): Sized,
// 		R: Sized,
// 	{
// 		Ok(events.find_first::<Ev>()?.map(|event| handler(event)))
// 	}
// }
