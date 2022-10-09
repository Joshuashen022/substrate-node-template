use node_template_runtime::{
	AccountId, BabeConfig, BalancesConfig, GenesisConfig, Signature, SudoConfig, SessionConfig,
	SystemConfig, WASM_BINARY, opaque::SessionKeys
};
// use pallet_session::pallet::GenesisConfig;
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{sr25519, Pair, Public};

use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

mod babe_genesis{
	const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
	pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
		sp_consensus_babe::BabeEpochConfiguration {
			c: PRIMARY_PROBABILITY,
			allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots
		};
}

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
pub fn authority_keys_from_seed(s: &str) -> BabeId {
	get_from_seed::<BabeId>(s)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	println!("using development {}", line!());
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
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
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
		// Extensions
		None,
	))
}

fn session_keys(babe: BabeId) -> SessionKeys {
	SessionKeys { babe}
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
			// println!("calling (testnet_genesis) {}", line!());
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
					authority_keys_from_seed("Charlie"),
					authority_keys_from_seed("Dave"),
					authority_keys_from_seed("Eve"),
					authority_keys_from_seed("Ferdie"),
					authority_keys_from_seed("Gabe"),
					authority_keys_from_seed("Hadley"),
					authority_keys_from_seed("Ian"),
					authority_keys_from_seed("Jack"),
					authority_keys_from_seed("Karen"),
					authority_keys_from_seed("Ferdie"),
					authority_keys_from_seed("Lacey"),
					authority_keys_from_seed("Monica"),
					authority_keys_from_seed("Nancy"),
					authority_keys_from_seed("Oliver"),
					authority_keys_from_seed("Pam"),
					authority_keys_from_seed("Quinn"),
					authority_keys_from_seed("Ross"),
					authority_keys_from_seed("Sabrina"),
				],
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
					get_account_id_from_seed::<sr25519::Public>("Gabe"),
					get_account_id_from_seed::<sr25519::Public>("Hadley"),
					get_account_id_from_seed::<sr25519::Public>("Ian"),
					get_account_id_from_seed::<sr25519::Public>("Jack"),
					get_account_id_from_seed::<sr25519::Public>("Karen"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Lacey"),
					get_account_id_from_seed::<sr25519::Public>("Monica"),
					get_account_id_from_seed::<sr25519::Public>("Nancy"),
					get_account_id_from_seed::<sr25519::Public>("Oliver"),
					get_account_id_from_seed::<sr25519::Public>("Pam"),
					get_account_id_from_seed::<sr25519::Public>("Quinn"),
					get_account_id_from_seed::<sr25519::Public>("Ross"),
					get_account_id_from_seed::<sr25519::Public>("Sabrina"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Gabe//stash"),
					get_account_id_from_seed::<sr25519::Public>("Hadley//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ian//stash"),
					get_account_id_from_seed::<sr25519::Public>("Jack//stash"),
					get_account_id_from_seed::<sr25519::Public>("Karen//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Lacey//stash"),
					get_account_id_from_seed::<sr25519::Public>("Monica//stash"),
					get_account_id_from_seed::<sr25519::Public>("Nancy//stash"),
					get_account_id_from_seed::<sr25519::Public>("Oliver//stash"),
					get_account_id_from_seed::<sr25519::Public>("Pam//stash"),
					get_account_id_from_seed::<sr25519::Public>("Quinn//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ross//stash"),
					get_account_id_from_seed::<sr25519::Public>("Sabrina//stash"),
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
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<BabeId>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	use sp_consensus_babe::BabeAuthorityWeight;
	// println!("(testnet_genesis)");
	let mut authorities:Vec<(BabeId,BabeAuthorityWeight)> = Vec::new();
	for auth in initial_authorities.clone(){
		let stake:BabeAuthorityWeight = 100;
		authorities.push((auth,stake));
	}

	let mut sessionkeys = Vec::new();
	for (account, key) in endowed_accounts.iter().zip(initial_authorities){
		sessionkeys.push((account.clone(), account.clone(), session_keys(key.clone())));
	}

	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		babe: BabeConfig {
			authorities, // pub authorities: Vec<(AuthorityId, BabeAuthorityWeight)>,
			epoch_config: Some(babe_genesis::BABE_GENESIS_EPOCH_CONFIG),
		},
		session: SessionConfig {
			keys: sessionkeys,
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
	}
}
