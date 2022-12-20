use sp_core::H256;
use subxt::{events::Events, OnlineClient};

use crate::kilt::KiltConfig;

pub mod eth;

/// Traverses back blocks and applies `event_handler` until reaching genesis or
/// the specified start hash. The latter could be the block of the runtime
/// upgrade.
///
/// NOTE: For local chains, this only works for the last 258 blocks since
/// collators are not a full node.
async fn filter_blocks_for_event<R, F: Fn(&Events<KiltConfig>) -> anyhow::Result<Vec<R>>>(
	api: OnlineClient<KiltConfig>,
	start_hash: Option<H256>,
	event_handler: F,
) -> anyhow::Result<Vec<R>> {
	let mut block = api
		.blocks()
		.at(None)
		.await
		.unwrap_or_else(|_| panic!("Failed to retrieve latest block"));

	let mut v = vec![];
	while Some(block.hash()) != start_hash && block.number() > 0 {
		println!("Inside block {:?} with hash {:?}", block.number(), block.hash());
		let events = block.events().await?;
		v.append(&mut event_handler(&events)?);

		block = api
			.blocks()
			.at(block.header().parent_hash.into())
			.await
			.unwrap_or_else(|_| panic!("Failed to retrieve parent block for {:?}", block.hash()));
	}

	Ok(v)
}
