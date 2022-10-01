use derivative::Derivative;
use parity_scale_codec::{Compact, Encode};
use sp_runtime::generic::Era;
use subxt::{
    tx::{ExtrinsicParams, PlainTip},
    utils::Encoded,
    Config,
};

#[derive(Derivative)]
#[derivative(Debug())]
pub struct KiltExtrinsicParams<T: Config> {
    era: Era,
    nonce: T::Index,
    tip: PlainTip,
    spec_version: u32,
    transaction_version: u32,
    genesis_hash: T::Hash,
    mortality_checkpoint: T::Hash,
    marker: std::marker::PhantomData<T>,
}

#[derive(Derivative)]
#[derivative(Debug())]
pub struct KiltExtrinsicParamsBuilder<T: Config> {
    era: Era,
    mortality_checkpoint: Option<T::Hash>,
    tip: PlainTip,
    spec_version: Option<u32>,
    transaction_version: Option<u32>,
}

impl<T: Config> KiltExtrinsicParamsBuilder<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn era(mut self, era: Era, checkpoint: T::Hash) -> Self {
        self.era = era;
        self.mortality_checkpoint = Some(checkpoint);
        self
    }

    pub fn tip(mut self, tip: impl Into<PlainTip>) -> Self {
        self.tip = tip.into();
        self
    }

    pub fn spec_version(mut self, spec_version: u32) -> Self {
        self.spec_version = spec_version.into();
        self
    }

    pub fn transaction_version(mut self, transaction_version: u32) -> Self {
        self.transaction_version = transaction_version.into();
        self
    }
}

impl<T: Config> Default for KiltExtrinsicParamsBuilder<T> {
    fn default() -> Self {
        Self {
            era: Era::Immortal,
            mortality_checkpoint: None,
            tip: PlainTip::default(),
            spec_version: None,
            transaction_version: None,
        }
    }
}

impl<T: Config> ExtrinsicParams<T::Index, T::Hash> for KiltExtrinsicParams<T> {
    type OtherParams = KiltExtrinsicParamsBuilder<T>;

    fn new(
        // Provided from subxt client:
        spec_version: u32,
        transaction_version: u32,
        nonce: T::Index,
        genesis_hash: T::Hash,
        // Provided externally:
        other_params: Self::OtherParams,
    ) -> Self {
        KiltExtrinsicParams {
            era: other_params.era,
            mortality_checkpoint: other_params.mortality_checkpoint.unwrap_or(genesis_hash),
            tip: other_params.tip,
            nonce,
            spec_version: other_params.spec_version.unwrap_or(spec_version),
            transaction_version: other_params
                .transaction_version
                .unwrap_or(transaction_version),
            genesis_hash,
            marker: std::marker::PhantomData,
        }
    }

    fn encode_extra_to(&self, v: &mut Vec<u8>) {
        let nonce: u64 = self.nonce.into();
        let tip = Encoded(self.tip.encode());
        (self.era, Compact(nonce), tip).encode_to(v);
    }

    fn encode_additional_to(&self, v: &mut Vec<u8>) {
        (
            self.spec_version,
            self.transaction_version,
            self.genesis_hash,
            self.mortality_checkpoint,
        )
            .encode_to(v);
    }
}
