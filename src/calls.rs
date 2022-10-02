use crate::spiritnet::{
    self,
    runtime_types::spiritnet_runtime::Call as Pallet,
    runtime_types::{
        self as Pallets,
        pallet_democracy::vote::{AccountVote, Vote},
    },
};

use hex_literal::hex;
use sp_core::H256;
use subxt::tx::TxPayload;

/// Proposal hash for the referendum (parachain_system > authorize_code("0x1c46dd62730a80d3da9d43bd544ca30e32a3654a58edc9df0517249b5708b6c1"))
const REF_PROPOSAL_HASH: H256 = H256(hex!(
    "f26acc7507a206e050f96e4f010c4af8547fca163244a03a3635308104b44bae"
));
/// The call bytes that should be executed parachain_system > authorize_code("0x1c46dd62730a80d3da9d43bd544ca30e32a3654a58edc9df0517249b5708b6c1")
const REF_PROPOSAL_CALL: &[u8] =
    &hex!("50021c46dd62730a80d3da9d43bd544ca30e32a3654a58edc9df0517249b5708b6c1");
const PROPOSAL_WEIGHT: u64 = 900_000_000;
const PROPOSAL_LENGTH: u32 = 100;

const COUNCIL_PROPOSAL_INDEX: u32 = 28;
const TC_PROPOSAL_INDEX: u32 = 11;
const REFERENDA_INDEX: u32 = 11;
const WASM_BLOB: &[u8] = include_bytes!("../artifacts/spiritnet-1.7.3-1.wasm");

pub fn preimage() -> Box<dyn TxPayload> {
    Box::new(
        spiritnet::tx()
            .democracy()
            .note_preimage(REF_PROPOSAL_CALL.to_vec()),
    )
}

pub fn propose_external() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().propose(
        6,
        Pallet::Democracy(
            Pallets::pallet_democracy::pallet::Call::external_propose_majority {
                proposal_hash: REF_PROPOSAL_HASH,
            },
        ),
        40,
    ))
}

pub fn vote_motion() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().vote(
        REF_PROPOSAL_HASH, //FIXME
        COUNCIL_PROPOSAL_INDEX,
        true,
    ))
}

pub fn close_motion() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().close(
        REF_PROPOSAL_HASH, //FIXME
        COUNCIL_PROPOSAL_INDEX,
        PROPOSAL_WEIGHT,
        PROPOSAL_LENGTH,
    ))
}

pub fn fast_track() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().technical_committee().propose(
        5,
        Pallet::Democracy(Pallets::pallet_democracy::pallet::Call::fast_track {
            proposal_hash: REF_PROPOSAL_HASH,
            voting_period: 1,
            delay: 1,
        }),
        100,
    ))
}

pub fn vote_fast_track() -> Box<dyn TxPayload> {
    Box::new(
        spiritnet::tx()
            .technical_committee()
            .vote(REF_PROPOSAL_HASH, TC_PROPOSAL_INDEX, true),
    )
}

pub fn close_fast_track() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().technical_committee().close(
        REF_PROPOSAL_HASH,
        TC_PROPOSAL_INDEX,
        PROPOSAL_WEIGHT,
        PROPOSAL_LENGTH,
    ))
}

pub fn vote_referenda() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().democracy().vote(
        REFERENDA_INDEX,
        AccountVote::Standard {
            vote: Vote(1 | 0b1000_0000),
            balance: 5_000_000_000_000,
        },
    ))
}

pub fn enact_upgrade() -> Box<dyn TxPayload> {
    Box::new(
        spiritnet::tx()
            .parachain_system()
            .enact_authorized_upgrade(WASM_BLOB.to_vec()),
    )
}
