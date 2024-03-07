
use super::*;

pub type SystemTokenAssetIdOf<T> = <<T as Config>::Fungibles as Inspect>::AssetId;
pub type SystemTokenBalanceOf<T> = <<T as Config>::Fungibles as Insepct>::Balance;
pub type SystemTokenWeightOf<T> = <<T as Config>::Fungibles as InspectSystemToken>::SystemTokenWeight;

#[derive(Encode, Decode)]
pub enum ParachainRuntimePallets {
	#[codec(index = 2)]
	InfraParaCore(ParachainConfigCalls),
}

#[derive(Encode, Decode)]
pub enum ParachainConfigCalls {
	#[codec(index = 1)]
	UpdateFeeTable(Vec<u8>, Vec<u8>, SystemTokenBalance),
	#[codec(index = 2)]
	UpdateParaFeeRate(SystemTokenWeight),
	#[codec(index = 3)]
	UpdateRuntimeState,
	#[codec(index = 4)]
	RegisterSystemToken(SystemTokenAssetId, SystemTokenWeight),
	#[codec(index = 5)]
	CreateWrappedLocal(
		SystemTokenAssetId,
		Fiat,
		SystemTokenBalance,
		Vec<u8>,
		Vec<u8>,
		u8,
		SystemTokenWeight,
		u8,
		MultiLocation,
	),
	#[codec(index = 6)]
	DeregisterSystemToken(SystemTokenAssetId, bool),
}
