use crate::kilt::{
	runtime_types::frame_support::weights::weight_v2::Weight, sudo::calls::SudoUncheckedWeight, KiltConfig,
};
use subxt::{tx::StaticTxPayload, OnlineClient};

fn set_code(wasm: &[u8]) -> StaticTxPayload<SudoUncheckedWeight> {
	crate::kilt::tx().sudo().sudo_unchecked_weight(
		crate::kilt::RuntimeCall::System(crate::kilt::runtime_types::frame_system::pallet::Call::set_code {
			code: wasm.to_vec(),
		}),
		Weight {
			ref_time: 1_000_000_000,
			// wproof_size: 0,
		},
	)
}

pub async fn execute_set_code(
	api: OnlineClient<KiltConfig>,
	wasm_blob: &[u8],
	sudo_pair: sp_keyring::sr25519::sr25519::Pair,
) -> anyhow::Result<Option<sp_core::H256>> {
	let sudo_signer = subxt::tx::PairSigner::new(sudo_pair);

	println!("Preparung runtime upgrade via sudo.set_code");
	let upgrade_res = api
		.tx()
		.sign_and_submit_then_watch_default(&crate::extrinsics::upgrade::set_code(wasm_blob), &sudo_signer)
		.await?;
	let post_upgrade_block = upgrade_res.wait_for_in_block().await.unwrap();
	println!(
		"Runtime upgrade with sudo.set_code in block {} with extrinsic hash {}",
		post_upgrade_block.block_hash(),
		post_upgrade_block.extrinsic_hash()
	);

	Ok(Some(post_upgrade_block.block_hash()))
}
