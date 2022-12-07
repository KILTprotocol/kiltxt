#[cfg(feature = "pre-eth-migration")]
pub mod pre {

    use bip39::{Language, Mnemonic, MnemonicType};
    use sp_keyring::AccountKeyring;
    use subxt::ext::sp_runtime::SaturatedConversion;
    use subxt::{
        ext::{sp_core::Pair, sp_runtime::app_crypto::sr25519},
        tx::PairSigner,
        OnlineClient,
    };

    use crate::extrinsics;

    const MAX_DID_BATCH_SIZE: usize = 100;

    fn create_tmp_keypair() -> anyhow::Result<sp_keyring::sr25519::sr25519::Pair> {
        // create new DID saving seed in the seeds vector
        let m_type = MnemonicType::for_word_count(12)?;
        let mnemonic = Mnemonic::new(m_type, Language::English);
        let m: String = mnemonic.phrase().into();
        sr25519::Pair::from_string_with_seed(&m, None)
            .map(|(pair, _)| pair)
            .or_else(|_| {
                anyhow::bail!(
                    "Failed to create sr25519 keypair from random mnemonic {}",
                    mnemonic
                )
            })
    }

    pub async fn spawn_linked_dids(
        api: OnlineClient<crate::kilt::KiltConfig>,
        num_dids: u32,
    ) -> anyhow::Result<()> {
        println!("Spawning {} new DIDs and linking to Alice", num_dids);

        let alice_pair = AccountKeyring::Alice.pair();
        let alice = PairSigner::new(alice_pair.clone());
        let current_block = api.blocks().at(None).await.unwrap().number();

        // build batch calls
        let mut calls = vec![];
        let mut handles = vec![];
        for i in 0..num_dids {
            let keypair = create_tmp_keypair()?;

            // create batch tx whenever max batch size is reached or end of loop
            if calls.len() == MAX_DID_BATCH_SIZE || i == num_dids - 1 {
                println!(
                    "[#{}/{}] Preparing DID create + link account batch call of size {}",
                    i / MAX_DID_BATCH_SIZE.saturated_into::<u32>() + 1,
                    num_dids / MAX_DID_BATCH_SIZE.saturated_into::<u32>() + 1,
                    calls.len()
                );

                let tx = crate::kilt::tx().utility().batch(calls);
                calls = vec![];

                let res = api
                    .tx()
                    .sign_and_submit_then_watch_default(&tx, &alice)
                    .await?;

                handles.push(res);
            }

            // create dummy DID
            calls.push(extrinsics::did::dummy_create_did(
                alice_pair.public(),
                keypair.clone(),
            ));

            // link dummy DID to Alice's address
            calls.push(extrinsics::did::dummy_link_account_with_did(
                alice_pair.public(),
                keypair.clone(),
                current_block,
            ));
        }

        // submit batch calls
        for handle in handles {
            let res = handle.wait_for_in_block().await.unwrap();
            println!("Done with batch in extrinsic {}", res.extrinsic_hash());
        }

        Ok(())
    }
}
