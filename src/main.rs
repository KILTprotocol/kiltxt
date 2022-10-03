use clap::Parser;
use extrinsic_param::KiltExtrinsicParams;
use sp_runtime::app_crypto::Pair;
use std::fs;
use subxt::tx::{Era, PlainTip};
use subxt::{tx::PairSigner, Config, OnlineClient};

use crate::extrinsic_param::KiltExtrinsicParamsBuilder;

#[subxt::subxt(runtime_metadata_path = "./artifacts/metadata/spirit-10730-rescue.scale")]
pub mod spiritnet {}

pub enum KiltConfig {}

mod calls;
mod extrinsic_param;

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

    /// mnemonic file path
    #[arg(short, long)]
    tip: Option<u128>,

    /// mnemonic file path
    #[arg(short, long)]
    call: CallSelect,

    /// mnemonic file path
    #[arg(short, long, default_value = "false")]
    send: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum CallSelect {
    Preimage,
    Propose,
    VoteMotion,
    CloseMotion,
    FastTack,
    VoteFastTrack,
    CloseFastTrack,
    VoteReferenda,
    EnactUpgrade,
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

    let tx = match args.call {
        CallSelect::Preimage => calls::preimage(),
        CallSelect::Propose => calls::propose_external(),
        CallSelect::VoteMotion => calls::vote_motion(),
        CallSelect::CloseMotion => calls::close_motion(),
        CallSelect::FastTack => calls::fast_track(),
        CallSelect::VoteFastTrack => calls::vote_fast_track(),
        CallSelect::CloseFastTrack => calls::close_fast_track(),
        CallSelect::VoteReferenda => calls::vote_referenda(),
        CallSelect::EnactUpgrade => calls::enact_upgrade(),
    };

    let mut params = KiltExtrinsicParamsBuilder::new()
        .era(Era::Immortal, api.genesis_hash())
        .spec_version(10101)
        .transaction_version(1)
        .nonce(args.nonce);

    if let Some(tip) = args.tip {
        params = params.tip(PlainTip::new(tip));
    }

    // Submit the transaction with default params:
    let tx = api.tx().create_signed(tx.as_ref(), &signer, params).await?;

    println!("signed `0x{}`", hex::encode(tx.encoded()));

    if args.send {
        tx.submit_and_watch()
            .await?
            .wait_for_finalized_success()
            .await?;
    }

    Ok(())
}
