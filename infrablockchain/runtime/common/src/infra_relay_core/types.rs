
use super::*;

#[derive(Encode, Decode)]
pub enum ParachainRuntimePallets {
	#[codec(index = 2)]
	InfraParaCore(ParachainConfigCalls),
}

#[derive(Encode, Decode)]
pub enum ParachainConfigCalls {
	#[codec(index = 0)]
	UpdateInfraSystemConfig(InfraSystemConfig),
	#[codec(index = 1)]
	SetFeeTable(Vec<u8>, Vec<u8>, SystemTokenBalance),
	#[codec(index = 2)]
	SetParaFeeRate(SystemTokenWeight),
	#[codec(index = 3)]
	SetRuntimeState,
	#[codec(index = 4)]
	UpdateSystemTokenWeight(SystemTokenAssetId, SystemTokenWeight),
	#[codec(index = 5)]
	RegisterSystemToken(SystemTokenAssetId, SystemTokenWeight),
	#[codec(index = 6)]
	CreateWrappedLocal(
		SystemTokenAssetId,
		Fiat,
		SystemTokenBalance,
		Vec<u8>,
		Vec<u8>,
		u8,
		SystemTokenWeight,
		u8,
		SystemTokenId,
	),
	#[codec(index = 7)]
	DeregisterSystemToken(SystemTokenAssetId, bool),
}
