/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG

	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0

	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.

*/

//! Defines all concrete types and global components of the enclave.
//!
//! This allows the crates themselves to stay as generic as possible
//! and ensures that the global instances are initialized once.

use crate::ocall::OcallApi;
use itc_parentchain::{
	block_import_dispatcher::immediate_dispatcher::ImmediateDispatcher,
	block_importer::ParentchainBlockImporter, light_client::ValidatorAccessor,
};
use itp_component_container::ComponentContainer;
use itp_extrinsics_factory::ExtrinsicsFactory;
use itp_nonce_cache::NonceCache;
use itp_types::Block as ParentchainBlock;
use sp_core::ed25519::Pair;

pub type EnclaveExtrinsicsFactory = ExtrinsicsFactory<Pair, NonceCache>;
pub type EnclaveValidatorAccessor = ValidatorAccessor<ParentchainBlock>;
pub type EnclaveParentChainBlockImporter = ParentchainBlockImporter<
	ParentchainBlock,
	EnclaveValidatorAccessor,
	OcallApi,
	EnclaveExtrinsicsFactory,
>;
pub type EnclaveParentchainBlockImportImmediateDispatcher =
	ImmediateDispatcher<EnclaveParentChainBlockImporter>;

pub static GLOBAL_PARENTCHAIN_IMPORT_IMMEDIATE_DISPATCHER_COMPONENT: ComponentContainer<
	EnclaveParentchainBlockImportImmediateDispatcher,
> = ComponentContainer::new();
