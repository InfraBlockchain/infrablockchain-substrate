
use super::*;

#[derive(Encode, Decode)]
pub enum ParachainRuntimePallets {
    #[codec(index = 2)]
    ParachainConfig(ParachainConfigCalls)
}

#[derive(Encode, Decode)]
pub enum ParachainConfigCalls {
    #[codec(index = 0)]
    SetBaseWeight,
    #[codec(index = 1)]
    SetFeeTable(Vec<u8>, Vec<u8>, SystemTokenBalance),
    #[codec(index = 2)]
    SetFeeRate(SystemTokenWeight),
    #[codec(index = 3)]
    SetRuntimeState,
    #[codec(index = 4)]
    SetSystemTokenWeight(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 5)]
    Register(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 6)]
    Create(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 7)]
    Deregister(SystemTokenAssetId),
}