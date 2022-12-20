use crate::{
	kilt,
	kilt::{kilt_runtime, sudo::calls::SudoUncheckedWeight, KiltConfig},
};
use subxt::{tx::StaticTxPayload, OnlineClient};

fn set_code(wasm: &[u8]) -> StaticTxPayload<SudoUncheckedWeight> {
	#[cfg(all(feature = "10801", not(feature = "default")))]
	{
		kilt::tx().sudo().sudo_unchecked_weight(
			kilt_runtime::Call::System(kilt::runtime_types::frame_system::pallet::Call::set_code {
				code: wasm.to_vec(),
			}),
			kilt::runtime_types::frame_support::weights::weight_v2::Weight {
				ref_time: 1_000_000_000,
			},
		)
	}
	kilt::tx().sudo().sudo_unchecked_weight(
		kilt_runtime::RuntimeCall::System(kilt::runtime_types::frame_system::pallet::Call::set_code {
			code: wasm.to_vec(),
		}),
		kilt::runtime_types::sp_weights::weight_v2::Weight {
			ref_time: 1_000_000_000,
			proof_size: 0,
		},
	)
}
pub async fn execute_set_code(
	api: OnlineClient<KiltConfig>,
	wasm_blob: &[u8],
	sudo_pair: sp_keyring::sr25519::sr25519::Pair,
) -> anyhow::Result<()> {
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

	// TODO: Feat: Move to EventHandler trait
	if post_upgrade_block
		.fetch_events()
		.await?
		.find_first::<kilt::sudo::events::Sudid>()?
		.is_none()
	{
		println!("Sudo call failed")
	}

	Ok(())
}
