///! Integritee worker. Inspiration for this design came from parity's substrate Client.
///
/// This should serve as a proof of concept for a potential refactoring design. Ultimately, everything
/// from the main.rs should be covered by the worker struct here - hidden and split across
/// multiple traits.
use std::sync::Arc;

pub struct Worker<Config, NodeApi, Enclave> {
	_config: Config,
	node_api: NodeApi, // todo: Depending on system design, all the api fields should be Arc<Api>
	// unused yet, but will be used when more methods are migrated to the worker
	_enclave_api: Arc<Enclave>,
}

impl<Config, NodeApi, Enclave> Worker<Config, NodeApi, Enclave> {
	pub fn new(_config: Config, node_api: NodeApi, _enclave_api: Arc<Enclave>) -> Self {
		Self { _config, node_api, _enclave_api }
	}

	// will soon be used.
	#[allow(dead_code)]
	pub fn node_api(&self) -> &NodeApi {
		&self.node_api
	}
}
