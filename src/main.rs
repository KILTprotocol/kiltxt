mod extrinsic_param;

use clap::Parser;
use extrinsic_param::KiltExtrinsicParams;
use hex_literal::hex;
use sp_core::H256;
use sp_runtime::app_crypto::Pair;
use std::fs;
use subxt::tx::Era;
use subxt::{tx::PairSigner, Config, OnlineClient};

use spiritnet::runtime_types as Pallets;
use spiritnet::runtime_types::spiritnet_runtime::Call as Pallet;

use crate::extrinsic_param::KiltExtrinsicParamsBuilder;

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
    type ExtrinsicParams = KiltExtrinsicParams<Self>;
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

    spiritnet::validate_codegen(&api)?;

    log::info!("Version {:#?}", api.runtime_version());

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
    let tx = api
        .tx()
        .create_signed(
            &tx,
            &signer,
            KiltExtrinsicParamsBuilder::new()
                .era(Era::Immortal, api.genesis_hash())
                .spec_version(10110)
                .transaction_version(1),
        )
        .await?;

    println!("signed `0x{}`", hex::encode(tx.encoded()));

    Ok(())
}
