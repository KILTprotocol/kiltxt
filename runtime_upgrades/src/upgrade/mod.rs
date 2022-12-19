use anyhow::ensure;
use sp_core::{crypto::AccountId32, H256};
use subxt::OnlineClient;

use crate::{
	kilt,
	kilt::{runtime_types::parachain_staking::types::Stake, KiltConfig},
};

/// Explicit checking of specific storage keys pre and post upgrade.
///
/// NOTE: In subxt v0.25.0 this cannot not be done by iterating overa generic
/// Vec<StorageAddress> because of different resulting storage types.
/// TODO: Move checks to separate functions or Trait.
pub async fn post_upgrade_sanity_checks(
	api: OnlineClient<KiltConfig>,
	now: Option<H256>,
	pre_upgrade_block_hash: Option<H256>,
) -> anyhow::Result<()> {
	if pre_upgrade_block_hash.is_none() {
		println!("Comparison with pre upgrade state skipped since block hash was not provided");
	}

	// Session Queuey Keys (must never be empty)
	println!("Post upgrade check for Session Queued Keys");
	let storage_key = kilt::storage().session().queued_keys();
	let current = api.storage().fetch(&storage_key, now).await?;
	ensure!(current.is_some(), "Post upgrade empty storage: session.queued_keys");
	let curr_ids: Vec<AccountId32> = current.unwrap().into_iter().map(|(acc_id, _)| acc_id).collect();
	let old_ids: Vec<AccountId32> = api
		.storage()
		.fetch(&storage_key, pre_upgrade_block_hash)
		.await?
		.unwrap()
		.into_iter()
		.map(|(acc_id, _)| acc_id)
		.collect();
	ensure!(
		curr_ids == old_ids,
		"Pre and post upgrade mismatch: session.queued_keys"
	);

	// Staking Top Candidates (must never be empty)
	println!("Post upgrade check for Staking Top Candidates");
	let storage_key = kilt::storage().parachain_staking().top_candidates();
	let current = api.storage().fetch(&storage_key, now).await?;
	ensure!(
		current.is_some(),
		"Post upgrade empty storage: parachain_staking.top_candidates"
	);

	let curr_ids: Vec<AccountId32> = current
		.unwrap()
		// Retrieve BoundedVec (trait impls not available, see https://github.com/paritytech/subxt/issues/545)
		.0
		// Retrieve Vec (trait impls not available, see https://github.com/paritytech/subxt/issues/545)
		 .0
		.into_iter()
		.map(|Stake { owner, .. }| owner)
		.collect();
	let old_ids: Vec<AccountId32> = api
		.storage()
		.fetch(&storage_key, pre_upgrade_block_hash)
		.await?
		.unwrap()
		// Retrieve BoundedVec (trait impls not available, see https://github.com/paritytech/subxt/issues/545)
		.0
		// Retrieve Vec (trait impls not available, see https://github.com/paritytech/subxt/issues/545)
		 .0
		.into_iter()
		.map(|Stake { owner, .. }| owner)
		.collect();
	ensure!(
		curr_ids == old_ids,
		"Pre and post upgrade mismatch: session.queued_keys"
	);

	// Council (must only be empty for dev chains)
	println!("Post upgrade check for Council Members");
	let storage_key = kilt::storage().council().members();
	let current_ids = api.storage().fetch(&storage_key, now).await?.unwrap_or_default();
	// Only soft check as dev chains have empty Council
	println!("Post upgrade council size is {:?}", current_ids.len());
	let old_ids = api
		.storage()
		.fetch(&storage_key, pre_upgrade_block_hash)
		.await?
		.unwrap_or_default();
	ensure!(current_ids == old_ids, "Pre and post upgrade mismatch: council.members");

	// Technical Committee (must only be empty for dev chains)
	println!("Post upgrade check for Technical Committee");
	let storage_key = kilt::storage().technical_committee().members();
	let current_ids = api.storage().fetch(&storage_key, now).await?.unwrap_or_default();
	// Only soft check as dev chains have empty Technical Committee
	println!("Post upgrade Technical Committee size is {:?}", current_ids.len());
	let old_ids = api
		.storage()
		.fetch(&storage_key, pre_upgrade_block_hash)
		.await?
		.unwrap_or_default();
	ensure!(
		current_ids == old_ids,
		"Pre and post upgrade mismatch: technical_committee.members"
	);
	if !current_ids.is_empty() {
		let membership_ids = api
			.storage()
			.fetch(
				&kilt::storage().technical_membership().members(),
				pre_upgrade_block_hash,
			)
			.await?
			.unwrap()
			// Retrieve BoundedVec (trait impls not available, see https://github.com/paritytech/subxt/issues/545)
			.0;
		ensure!(
			current_ids == membership_ids,
			"Post upgrade mismatch: technical_committee.members != technical_membership.members"
		);
	}

	#[cfg(not(feature = "pre-eth-migration"))]
	{
		let storage_key = kilt::storage().did_lookup().migration_ongoing();
		let migration_ongoing = api.storage().fetch(&storage_key, None).await?;
		println!("Migration ongoing? {:?}", migration_ongoing);
		ensure!(
			migration_ongoing == Some(true),
			"Post ethereum migration flag is not set!"
		);

		// TODO: Add storage version
	}

	Ok(())
}

pub trait RuntimeUpgradeSanityChecker {
	fn check_session_queued_keys() -> bool;
	fn check_staking_top_candidates() -> bool;
	fn check_council_members() -> bool;
	fn check_technical_committee_members() -> bool;
	// TODO:
	// fn check_block_production
	// fn check_authorship_authors
	// fn check_democracy_referenda
}
