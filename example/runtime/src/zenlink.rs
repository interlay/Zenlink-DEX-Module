use core::convert::TryInto;

use super::*;

use sp_runtime::{DispatchError, DispatchResult};
use sp_std::marker::PhantomData;

use chain_primitives::*;
use zenlink_protocol::*;
use zenlink_stable_amm::traits::{StablePoolLpCurrencyIdGenerate, ValidateCurrency};

parameter_types! {
	pub SelfParaId: u32 = ParachainInfo::parachain_id().into();
	pub const ZenlinkPalletId: PalletId = PalletId(*b"/zenlink");
	pub ZenlinkRegistedParaChains: Vec<(MultiLocation, u128)> = vec![
		(make_x2_location(2001), 10_000_000_000),
	];
	pub const StringLimit: u32 = 50;
	pub const StableAmmPalletId: PalletId = PalletId(*b"bf/stamm");
}

impl zenlink_protocol::Config for Runtime {
	type Event = super::Event;
	type MultiAssetsHandler = MultiAssets;
	type PalletId = ZenlinkPalletId;
	type TargetChains = ZenlinkRegistedParaChains;
	type SelfParaId = SelfParaId;
	type Conversion = ();
	type XcmExecutor = ();
	type WeightInfo = ();
}

type MultiAssets = ZenlinkMultiAssets<ZenlinkProtocol, Balances, LocalAssetAdaptor<Tokens>>;

pub struct LocalAssetAdaptor<Local>(PhantomData<Local>);

impl<Local, AccountId> LocalAssetHandler<AccountId> for LocalAssetAdaptor<Local>
where
	Local: MultiCurrency<AccountId, CurrencyId = CurrencyId>,
{
	fn local_balance_of(asset_id: ZenlinkAssetId, who: &AccountId) -> AssetBalance {
		if let Ok(currency_id) = asset_id.try_into() {
			return TryInto::<AssetBalance>::try_into(Local::free_balance(currency_id, &who))
				.unwrap_or_default()
		}
		AssetBalance::default()
	}

	fn local_total_supply(asset_id: ZenlinkAssetId) -> AssetBalance {
		if let Ok(currency_id) = asset_id.try_into() {
			return TryInto::<AssetBalance>::try_into(Local::total_issuance(currency_id))
				.unwrap_or_default()
		}
		AssetBalance::default()
	}

	fn local_is_exists(asset_id: ZenlinkAssetId) -> bool {
		let currency_id: Result<CurrencyId, ()> = asset_id.try_into();
		match currency_id {
			Ok(_) => true,
			Err(_) => false,
		}
	}

	fn local_transfer(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		target: &AccountId,
		amount: AssetBalance,
	) -> DispatchResult {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::transfer(
				currency_id,
				&origin,
				&target,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local transfer"))?,
			)
		} else {
			Err(DispatchError::Other("unknown asset in local transfer"))
		}
	}

	fn local_deposit(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::deposit(
				currency_id,
				&origin,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local deposit"))?,
			)?;
		} else {
			return Err(DispatchError::Other("unknown asset in local transfer"))
		}

		Ok(amount)
	}

	fn local_withdraw(
		asset_id: ZenlinkAssetId,
		origin: &AccountId,
		amount: AssetBalance,
	) -> Result<AssetBalance, DispatchError> {
		if let Ok(currency_id) = asset_id.try_into() {
			Local::withdraw(
				currency_id,
				&origin,
				amount
					.try_into()
					.map_err(|_| DispatchError::Other("convert amount in local withdraw"))?,
			)?;
		} else {
			return Err(DispatchError::Other("unknown asset in local transfer"))
		}

		Ok(amount)
	}
}

type PoolId = u32;
impl zenlink_stable_amm::Config for Runtime {
	type Event = super::Event;
	type CurrencyId = CurrencyId;
	type MultiCurrency = Tokens;
	type PoolId = PoolId;
	type TimeProvider = Timestamp;
	type EnsurePoolAsset = StableAmmVerifyPoolAsset;
	type LpGenerate = PoolLpGenerate;
	type PoolCurrencySymbolLimit = StringLimit;
	type PalletId = StableAmmPalletId;
}

pub struct PoolLpGenerate;
impl StablePoolLpCurrencyIdGenerate<CurrencyId, PoolId> for PoolLpGenerate {
	fn generate_by_pool_id(pool_id: PoolId) -> CurrencyId {
		CurrencyId::StableLpToken(pool_id)
	}
}

pub struct StableAmmVerifyPoolAsset;
impl ValidateCurrency<CurrencyId> for StableAmmVerifyPoolAsset {
	fn validate_pooled_currency(_currencies: &[CurrencyId]) -> bool {
		true
	}

	fn validate_pool_lp_currency(_currency_id: CurrencyId) -> bool {
		if Tokens::total_issuance(_currency_id) > 0 {
			return false
		}
		true
	}
}
