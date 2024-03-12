use super::*;

pub type SystemTokenAssetIdOf<T> = <<T as Config>::Fungibles as Inspect>::AssetId;
pub type SystemTokenBalanceOf<T> = <<T as Config>::Fungibles as Insepct>::Balance;
pub type SystemTokenWeightOf<T> =
	<<T as Config>::Fungibles as InspectSystemToken>::SystemTokenWeight;

#[derive(Encode, Decode)]
pub enum ParachainRuntimePallets {
	#[codec(index = 2)]
	InfraParaCore(ParachainConfigCalls),
}

#[derive(Encode, Decode)]
pub enum ParachainConfigCalls<Location, Balance, Weight> {
	#[codec(index = 1)]
	UpdateFeeTable(Vec<u8>, Vec<u8>, Balance),
	#[codec(index = 2)]
	UpdateParaFeeRate(Weight),
	#[codec(index = 3)]
	UpdateRuntimeState,
	#[codec(index = 4)]
	RegisterSystemToken(Location, Weight),
	#[codec(index = 5)]
	CreateWrappedLocal(Location, Fiat, SystemTokenBalance, Vec<u8>, Vec<u8>, u8, SystemTokenWeight),
	#[codec(index = 6)]
	DeregisterSystemToken(Location, bool),
}
