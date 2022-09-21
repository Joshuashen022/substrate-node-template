//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use node_template_runtime::{self, opaque::Block, RuntimeApi};
use sc_client_api::ExecutorProvider;
use sc_consensus_babe:: {SlotProportion, calculate_current_slot};
pub use sc_executor::NativeElseWasmExecutor;
use sc_keystore::LocalKeystore;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use std::sync::Arc;
use sp_api::HeaderT;
// Our native executor instance.
pub struct ExecutorDispatch;
use sc_client_api::UsageProvider;

use sp_runtime::generic::BlockId;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		node_template_runtime::api::dispatch(method, data)
	}

	fn native_version() -> sc_executor::NativeVersion {
		node_template_runtime::native_version()
	}
}

type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

pub fn new_partial(
	config: &Configuration,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block, FullClient>,
		sc_transaction_pool::FullPool<Block, FullClient>,
		(
			sc_consensus_babe::BabeBlockImport<
				Block,
				FullClient,
				Arc<FullClient>,
			>,
			Option<sc_finality_grandpa::LinkHalf<Block, FullClient, FullSelectChain>>,
			Option<Telemetry>,
			sc_consensus_babe::BabeLink<Block>,
		),
	>,
	ServiceError
	>
{
	if config.keystore_remote.is_some() {
		return Err(ServiceError::Other(format!("Remote Keystores are not supported.")))
	}

	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
	);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			&config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::Config::get_or_compute(&*client)?,
		client.clone(),  // grandpa_block_import, TODO::here's the problem
		client.clone(),
	)?;

	let slot_duration = babe_link.config().slot_duration();
	let inherent_data_providers = move |_, ()| async move {
		let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

		let slot =
			sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
				*timestamp,
				slot_duration,
			);

		Ok((timestamp, slot))
	};
	log::info!("(new_partial) {}", line!());
	let import_queue = sc_consensus_babe::import_queue(
		babe_link.clone(),
		block_import.clone(),
		None, //Some(Box::new(justification_import)),
		client.clone(),
		select_chain.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_essential_handle(),
		config.prometheus_registry(),
		sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
		telemetry.as_ref().map(|x| x.handle()),
	)?;


	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, None, telemetry, babe_link),// TODO::here's the problem first None
	} )
}

fn remote_keystore(_url: &String) -> Result<Arc<LocalKeystore>, &'static str> {
	// FIXME: here would the concrete keystore be built,
	//        must return a concrete type (NOT `LocalKeystore`) that
	//        implements `CryptoStore` and `SyncCryptoStore`
	Err("Remote Keystore not supported.")
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		mut keystore_container,
		select_chain,
		transaction_pool,
		other: (block_import, _, mut telemetry, babe_link),
	} = new_partial(&config)?;

	if let Some(url) = &config.keystore_remote { // None
		match remote_keystore(url) {
			Ok(k) => keystore_container.set_remote_keystore(k),
			Err(e) =>
				return Err(ServiceError::Other(format!(
					"Error hooking up remote keystore for {}: {}",
					url, e
				))),
		};
	}

	let (network, system_rpc_tx, network_starter, adjusts_mutex, blocks_mutex) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync: None,
		})?;

	if config.offchain_worker.enabled {
		sc_service::build_offchain_workers(
			&config,
			task_manager.spawn_handle(),
			client.clone(),
			network.clone(),
		);
	}

	let client_clone = client.clone();
	let adjusts_mutex_clone = adjusts_mutex.clone();
	let test_future = async move {
		loop{
			std::thread::sleep(std::time::Duration::from_millis(6000));
			let engine_id = *b"ajst";
			let best_hash = client_clone.usage_info().chain.best_hash;
			if let Ok(headers) = client_clone.clone().header(&BlockId::hash(best_hash)){
				if let Some(hd) = headers {
					let _digest = hd.digest();

					// log::info!("Test Future get digest {:?}", digest);
				} else {
					log::info!("Test Future get no digest");
				}

			} else {
				log::info!("Test Future get no header");
			}

			if let Some(_adjust_raw) = client_clone.clone().adjusts_raw(engine_id, &BlockId::hash(best_hash)){
				log::info!("Test Future get some adjust_raw");
			} else {
				log::info!("Test Future get no adjust_raw");
			}
			if let Ok(guard) = adjusts_mutex_clone.clone().lock(){
				log::info!("adjusts_mutex len {}", (*guard).len());
			}
		}
	};
	task_manager.spawn_handle().spawn("Test Block", None,test_future);

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps =
				crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };

			Ok(crate::rpc::create_full(deps))
		})
	};
	// check if keystore has anything
	// keystore_container.local_keystore().unwrap().check_keys(); // No value
	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		client: client.clone(),
		backend,
		task_manager: &mut task_manager,
		keystore: keystore_container.sync_keystore(), // make local keystore contains value
		transaction_pool: transaction_pool.clone(),
		rpc_extensions_builder,
		network: network.clone(),
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;
	// check if keystore has anything
	// keystore_container.local_keystore().unwrap().check_keys(); // Has value

	if role.is_authority() {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool,
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let can_author_with =
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());
		let slot_duration = babe_link.config().slot_duration();

		//TODO:change this to autosyn inherent data provider
		let inherent_data_providers = move |_, ()| async move{
			let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

			let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_duration(
					*timestamp,
					slot_duration,
				);

			Ok((timestamp, slot))
		};
		let backoff_authoring_blocks: Option<()> = None;
		let auto_config = sc_consensus_babe::AutoSynParams {
			keystore: keystore_container.sync_keystore(),// this has no real effect, only return value
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: network.clone(),
			justification_sync_link: network.clone(),
			create_inherent_data_providers: inherent_data_providers.clone(),
			force_authoring,
			backoff_authoring_blocks, // error
			babe_link,
			can_author_with,
			block_proposal_slot_portion:SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion:None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			adjusts_mutex,
			blocks_mutex,
			task_manager: &mut task_manager,
		};

		let babe = sc_consensus_babe::start_autosyn(auto_config)?;

		// the AURA authoring task is considered essential, i.e. if it
		// fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking("babe-proposer", None, babe);
	}

	network_starter.start_network();
	Ok(task_manager)
}

