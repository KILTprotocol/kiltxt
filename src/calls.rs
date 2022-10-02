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

const PROPOSAL_HASH: H256 = H256(hex!(
    "af51c064817153d259ca698cd436264641da49074d0b30397e1d1834aeb2d033"
));
const PROPOSAL_WEIGHT: u64 = 900_000_000;
const PROPOSAL_LENGTH: u32 = 100;

const COUNCIL_PROPOSAL_INDEX: u32 = 28;
const TC_PROPOSAL_INDEX: u32 = 11;
const REFERENDA_INDEX: u32 = 11;

pub fn preimage() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().democracy().note_preimage(
        hex!("5002ead2ae37aa6611275f39efdabd292c6338f4ab50f8bea4e8a783b7fe39894e59").to_vec(),
    ))
}

pub fn propose_external() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().propose(
        6,
        Pallet::Democracy(
            Pallets::pallet_democracy::pallet::Call::external_propose_majority {
                proposal_hash: PROPOSAL_HASH,
            },
        ),
        40,
    ))
}

pub fn vote_motion() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().vote(
        PROPOSAL_HASH, //FIXME
        COUNCIL_PROPOSAL_INDEX,
        true,
    ))
}

pub fn close_motion() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().council().close(
        PROPOSAL_HASH, //FIXME
        COUNCIL_PROPOSAL_INDEX,
        PROPOSAL_WEIGHT,
        PROPOSAL_LENGTH,
    ))
}

pub fn fast_track() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().technical_committee().propose(
        5,
        Pallet::Democracy(Pallets::pallet_democracy::pallet::Call::fast_track {
            proposal_hash: PROPOSAL_HASH,
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
            .vote(PROPOSAL_HASH, TC_PROPOSAL_INDEX, true),
    )
}

pub fn close_fast_track() -> Box<dyn TxPayload> {
    Box::new(spiritnet::tx().technical_committee().close(
        PROPOSAL_HASH,
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
        spiritnet::tx().parachain_system().enact_authorized_upgrade(
            include_bytes!("../artifacts/spiritnet-1.7.3-1.wasm").to_vec(),
        ),
    )
}
