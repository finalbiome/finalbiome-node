use finalbiome_node_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, OrganizationIdentityConfig, FungibleAssetsConfig, Balance,
	pallet_fungible_assets::{AssetId as AssetId}
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
				],
				// Organization accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), br"Eve Org".to_vec()),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), br"Oscar Org".to_vec()),
				],
				// Organization Members
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Ferdie")),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Pat")),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), get_account_id_from_seed::<sr25519::Public>("Mike")),
				],
				// Fungible Assets
				vec![
					(0, get_account_id_from_seed::<sr25519::Public>("Eve"), br"ass01".to_vec(), None, None, None),
					(1, get_account_id_from_seed::<sr25519::Public>("Eve"), br"ass02".to_vec(), Some(5), None, Some(2000)),
				],
				// Balances of Fungible Assets
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 0, 1000),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 1, 20),
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
				],
				// Organization accounts
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), br"Eve Org".to_vec()),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), br"Oscar Org".to_vec()),
				],
				// Organization Members
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Ferdie")),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), get_account_id_from_seed::<sr25519::Public>("Pat")),
					(get_account_id_from_seed::<sr25519::Public>("Oscar"), get_account_id_from_seed::<sr25519::Public>("Mike")),
				],
				// Fungible Assets
				vec![
					(0, get_account_id_from_seed::<sr25519::Public>("Eve"), br"ass01".to_vec(), None, None, None),
					(1, get_account_id_from_seed::<sr25519::Public>("Eve"), br"ass02".to_vec(), Some(5), None, Some(2000)),
				],
				// Balances of Fungible Assets
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 0, 1000),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 1, 20),
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
	fungible_assets: Vec<(AssetId, AccountId, Vec<u8>, Option<Balance>, Option<Balance>, Option<Balance>)>,
	fungible_assets_balances: Vec<(AccountId, AssetId, Balance)>,
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
	}
}
