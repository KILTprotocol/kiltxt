// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! To run this example, a local polkadot node should be running. Example verified against polkadot v0.9.28-9ffe6e9e3da.
//!
//! E.g.
//! ```bash
//! curl "https://github.com/paritytech/polkadot/releases/download/v0.9.28/polkadot" --output /usr/local/bin/polkadot --location
//! polkadot --dev --tmp
//! ```

use clap::Parser;
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::app_crypto::Pair;
use std::fs;
use subxt::tx::Era;
use subxt::tx::SubstrateExtrinsicParamsBuilder;
use subxt::{tx::PairSigner, Config, OnlineClient};

use spiritnet::runtime_types as Pallets;
use spiritnet::runtime_types::spiritnet_runtime::Call as Pallet;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata/spirit-10730-rescue.scale")]
pub mod spiritnet {}

pub enum KiltConfig {}

impl Config for KiltConfig {
    type Index = u32;
    type BlockNumber = u64;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = sp_runtime::AccountId32;
    type Address = sp_runtime::MultiAddress<Self::AccountId, u32>;
    type Header = sp_runtime::generic::Header<Self::BlockNumber, sp_runtime::traits::BlakeTwo256>;
    type Signature = sp_runtime::MultiSignature;
    type Extrinsic = sp_runtime::OpaqueExtrinsic;
    type ExtrinsicParams = subxt::tx::SubstrateExtrinsicParams<Self>;
}

/// Simple program to create signed TX for specific runtime.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// websocket
    #[arg(short, long)]
    websocket: String,

    /// mnemonic file path
    #[arg(short, long)]
    mnemonic: String,

    /// mnemonic file path
    #[arg(short, long)]
    nonce: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let mnemonic = fs::read_to_string(args.mnemonic)?;
    let signer = PairSigner::new(
        sp_core::sr25519::Pair::from_string_with_seed(&mnemonic, None)
            .unwrap()
            .0,
    );

    // Create a client to use:
    let api = OnlineClient::<KiltConfig>::from_url(args.websocket).await?;

    // Create a transaction to submit:
    let tx = spiritnet::tx().council().propose(
        6,
        Pallet::ParachainSystem(
            Pallets::cumulus_pallet_parachain_system::pallet::Call::authorize_upgrade {
                code_hash: H256::from(&hex!(
                    "ead2ae37aa6611275f39efdabd292c6338f4ab50f8bea4e8a783b7fe39894e59"
                )),
            },
        ),
        40,
    );

    // Submit the transaction with default params:
    let signed_tx = api
        .tx()
        .create_signed(
            &tx,
            &signer,
            SubstrateExtrinsicParamsBuilder::new().era(Era::Immortal, api.genesis_hash()),
        )
        .await?;

    println!("submittable `0x{}`", hex::encode(signed_tx.encoded()));

    Ok(())
}
