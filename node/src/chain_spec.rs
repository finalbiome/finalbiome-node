use finalbiome_node_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, OrganizationIdentityConfig, FungibleAssetsConfig, NonFungibleAssetsConfig,
	pallet_fungible_assets::{AssetId as AssetId, AssetBalance},
	pallet_non_fungible_assets::{NonFungibleClassId, GenesisPurchasedClassesConfig},
};

use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Mike"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Mike//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
				],
				// Organization accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), br"WoW".to_vec()),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), br"Skyrim".to_vec()),
				],
				// Organization Members
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Ferdie")),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Pat")),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), get_account_id_from_seed::<sr25519::Public>("Mike")),
				],
				// Fungible Assets
				vec![
					(0.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"Gold".to_vec(), None, None, None),
					(1.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"Energy".to_vec(), Some(5.into()), None, Some(2000.into())),
				],
				// Balances of Fungible Assets
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 0.into(), 1000.into()),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 1.into(), 20.into()),
				],
				vec![
					(0.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"nfa01".to_vec()),
					(1.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"nfa02".to_vec()),
				],
				vec![
					(0.into(), br"atr1".to_vec(), 5u32, Some(10u32)),
					(0.into(), br"atr2".to_vec(), 20u32, None),
				],
				vec![
					(0.into(), br"atr3".to_vec(), br"stone".to_vec()),
				],
				vec![
					// class_id, fa_id, price, attrs [key, num_value, num_max, text_value]
					(0.into(), 0.into(), 10.into(), vec![(br"attr4".to_vec(), Some(10), None, None)]),
					(0.into(), 0.into(), 50.into(), vec![(br"attr4".to_vec(), Some(50), None, None)]),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
				],
				// Organization accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), br"WoW".to_vec()),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), br"Skyrim".to_vec()),
				],
				// Organization Members
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Ferdie")),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Pat")),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), get_account_id_from_seed::<sr25519::Public>("Mike")),
				],
				// Fungible Assets
				vec![
					(0.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"Gold".to_vec(), None, None, None),
					(1.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"Energy".to_vec(), Some(5.into()), None, Some(2000.into())),
				],
				// Balances of Fungible Assets
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 0.into(), 1000.into()),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 1.into(), 20.into()),
				],
				vec![
					(0.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"nfa01".to_vec()),
					(1.into(), get_account_id_from_seed::<sr25519::Public>("Eve"), br"nfa02".to_vec()),
				],
				vec![
					(0.into(), br"atr1".to_vec(), 5u32, Some(10u32)),
					(0.into(), br"atr2".to_vec(), 20u32, None),
				],
				vec![
					(0.into(), br"atr3".to_vec(), br"stone".to_vec()),
				],
				vec![
					// class_id, fa_id, price, attrs [key, num_value, num_max, text_value]
					(0.into(), 0.into(), 10.into(), vec![(br"attr4".to_vec(), Some(10), None, None)]),
					(0.into(), 0.into(), 50.into(), vec![(br"attr4".to_vec(), Some(50), None, None)]),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	organization_accounts: Vec<(AccountId, Vec<u8>)>,
	organization_members: Vec<(AccountId, AccountId)>,
	fungible_assets: Vec<(AssetId, AccountId, Vec<u8>, Option<AssetBalance>, Option<AssetBalance>, Option<AssetBalance>)>,
	fungible_assets_balances: Vec<(AccountId, AssetId, AssetBalance)>,
	non_fungible_classes: Vec<(NonFungibleClassId, AccountId, Vec<u8>)>,
	non_fungible_num_attributes: Vec<(NonFungibleClassId, Vec<u8>, u32, Option<u32>)>,
	non_fungible_text_attributes: Vec<(NonFungibleClassId, Vec<u8>, Vec<u8>)>,
	non_fungible_characteristics_purchased: GenesisPurchasedClassesConfig,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		organization_identity: OrganizationIdentityConfig {
			organizations: organization_accounts,
			members_of: organization_members,
		},
		fungible_assets: FungibleAssetsConfig {
			assets: fungible_assets,
			accounts: fungible_assets_balances,
		},
		non_fungible_assets: NonFungibleAssetsConfig {
			classes: non_fungible_classes,
			num_attributes: non_fungible_num_attributes,
			text_attributes: non_fungible_text_attributes,
			characteristics_purchased: non_fungible_characteristics_purchased,
		},
	}
}
