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
#![feature(structural_match)]
#![feature(rustc_attrs)]
#![feature(core_intrinsics)]
#![feature(derive_eq)]
#![feature(trait_alias)]
#![crate_name = "enclave_runtime"]
#![crate_type = "staticlib"]
#![cfg_attr(not(target_env = "sgx"), no_std)]
#![cfg_attr(target_env = "sgx", feature(rustc_private))]
#![allow(clippy::missing_safety_doc)]

#[cfg(not(target_env = "sgx"))]
#[macro_use]
extern crate sgx_tstd as std;

#[cfg(not(feature = "test"))]
use sgx_types::size_t;

use crate::{
	error::{Error, Result},
	global_components::{
		EnclaveValidatorAccessor, GLOBAL_PARENTCHAIN_IMPORT_IMMEDIATE_DISPATCHER_COMPONENT,
	},
	ocall::OcallApi,
	rpc::worker_api_direct::public_api_rpc_handler,
	utils::{hash_from_slice, write_slice_and_whitespace_pad, DecodeRaw},
};
use codec::{alloc::string::String, Decode, Encode};
use itc_direct_rpc_server::{
	create_determine_watch, rpc_connection_registry::ConnectionRegistry,
	rpc_ws_handler::RpcWsHandler,
};
use itc_parentchain::{
	block_import_dispatcher::{immediate_dispatcher::ImmediateDispatcher, DispatchBlockImport},
	block_importer::ParentchainBlockImporter,
	light_client::{concurrent_access::ValidatorAccess, LightClientState},
};
use itc_tls_websocket_server::{connection::TungsteniteWsConnection, run_ws_server};
use itp_component_container::{ComponentGetter, ComponentInitializer};
use itp_extrinsics_factory::ExtrinsicsFactory;
use itp_nonce_cache::{MutateNonce, Nonce, GLOBAL_NONCE_CACHE};
use itp_ocall_api::EnclaveAttestationOCallApi;
use itp_primitives_cache::GLOBAL_PRIMITIVES_CACHE;
use itp_settings::node::{
	REGISTER_ENCLAVE, RUNTIME_SPEC_VERSION, RUNTIME_TRANSACTION_VERSION, TEEREX_MODULE,
};
use itp_sgx_crypto::{aes, ed25519, rsa3072, Ed25519Seal, Rsa3072Seal};
use itp_sgx_io as io;
use itp_sgx_io::SealedIO;
use itp_storage::StorageProof;
use itp_types::{Block, Header, SignedBlock};
use log::*;
use sgx_types::sgx_status_t;
use sp_core::crypto::Pair;
use sp_finality_grandpa::VersionedAuthorityList;
use std::{slice, sync::Arc, vec::Vec};
use substrate_api_client::compose_extrinsic_offline;

mod attestation;
mod global_components;
mod ipfs;
mod ocall;
mod utils;

pub mod cert;
pub mod error;
pub mod rpc;
mod sync;
pub mod tls_ra;

#[cfg(feature = "test")]
pub mod test;

#[cfg(feature = "test")]
pub mod tests;

// this is a 'dummy' for production mode
#[cfg(not(feature = "test"))]
#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
	unreachable!("Tests are not available when compiled in production mode.")
}

pub const CERTEXPIRYDAYS: i64 = 90i64;

pub type Hash = sp_core::H256;
pub type AuthorityPair = sp_core::ed25519::Pair;

#[no_mangle]
pub unsafe extern "C" fn init(
	mu_ra_addr: *const u8,
	mu_ra_addr_size: u32,
	untrusted_worker_addr: *const u8,
	untrusted_worker_addr_size: u32,
) -> sgx_status_t {
	// Initialize the logging environment in the enclave.
	env_logger::init();

	if let Err(e) = ed25519::create_sealed_if_absent().map_err(Error::Crypto) {
		return e.into()
	}
	let signer = match Ed25519Seal::unseal().map_err(Error::Crypto) {
		Ok(pair) => pair,
		Err(e) => return e.into(),
	};
	info!("[Enclave initialized] Ed25519 prim raw : {:?}", signer.public().0);

	if let Err(e) = rsa3072::create_sealed_if_absent() {
		return e.into()
	}

	// Create the aes key that is used for state encryption such that a key is always present in tests.
	// It will be overwritten anyway if mutual remote attastation is performed with the primary worker.
	if let Err(e) = aes::create_sealed_if_absent().map_err(Error::Crypto) {
		return e.into()
	}

	let mu_ra_url =
		match String::decode(&mut slice::from_raw_parts(mu_ra_addr, mu_ra_addr_size as usize))
			.map_err(Error::Codec)
		{
			Ok(addr) => addr,
			Err(e) => return e.into(),
		};

	let untrusted_worker_url = match String::decode(&mut slice::from_raw_parts(
		untrusted_worker_addr,
		untrusted_worker_addr_size as usize,
	))
	.map_err(Error::Codec)
	{
		Ok(addr) => addr,
		Err(e) => return e.into(),
	};

	if let Err(e) = itp_primitives_cache::set_primitives(
		GLOBAL_PRIMITIVES_CACHE.as_ref(),
		&mu_ra_url,
		&untrusted_worker_url,
	)
	.map_err(Error::PrimitivesAccess)
	{
		return e.into()
	}

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn get_rsa_encryption_pubkey(
	pubkey: *mut u8,
	pubkey_size: u32,
) -> sgx_status_t {
	let rsa_pubkey = match Rsa3072Seal::unseal_pubkey() {
		Ok(key) => key,
		Err(e) => return e.into(),
	};

	let rsa_pubkey_json = match serde_json::to_string(&rsa_pubkey) {
		Ok(k) => k,
		Err(x) => {
			println!("[Enclave] can't serialize rsa_pubkey {:?} {}", rsa_pubkey, x);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
	write_slice_and_whitespace_pad(pubkey_slice, rsa_pubkey_json.as_bytes().to_vec());

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn get_ecc_signing_pubkey(pubkey: *mut u8, pubkey_size: u32) -> sgx_status_t {
	if let Err(e) = ed25519::create_sealed_if_absent().map_err(Error::Crypto) {
		return e.into()
	}

	let signer = match Ed25519Seal::unseal().map_err(Error::Crypto) {
		Ok(pair) => pair,
		Err(e) => return e.into(),
	};
	debug!("Restored ECC pubkey: {:?}", signer.public());

	let pubkey_slice = slice::from_raw_parts_mut(pubkey, pubkey_size as usize);
	pubkey_slice.clone_from_slice(&signer.public());

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn set_nonce(nonce: *const u32) -> sgx_status_t {
	log::info!("[Ecall Set Nonce] Setting the nonce of the enclave to: {}", *nonce);

	let mut nonce_lock = match GLOBAL_NONCE_CACHE.load_for_mutation() {
		Ok(l) => l,
		Err(e) => {
			error!("Failed to set nonce in enclave: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	*nonce_lock = Nonce(*nonce);

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn mock_register_enclave_xt(
	genesis_hash: *const u8,
	genesis_hash_size: u32,
	_nonce: *const u32,
	w_url: *const u8,
	w_url_size: u32,
	unchecked_extrinsic: *mut u8,
	unchecked_extrinsic_size: u32,
) -> sgx_status_t {
	let genesis_hash_slice = slice::from_raw_parts(genesis_hash, genesis_hash_size as usize);
	let genesis_hash = hash_from_slice(genesis_hash_slice);

	let mut url_slice = slice::from_raw_parts(w_url, w_url_size as usize);
	let url: String = Decode::decode(&mut url_slice).unwrap();
	let extrinsic_slice =
		slice::from_raw_parts_mut(unchecked_extrinsic, unchecked_extrinsic_size as usize);

	let mre = OcallApi
		.get_mrenclave_of_self()
		.map_or_else(|_| Vec::<u8>::new(), |m| m.m.encode());

	let signer = Ed25519Seal::unseal().unwrap();
	let call = ([TEEREX_MODULE, REGISTER_ENCLAVE], mre, url);

	let nonce_cache = GLOBAL_NONCE_CACHE.clone();
	let mut nonce_lock = nonce_cache.load_for_mutation().expect("Nonce lock poisoning");
	let nonce_value = nonce_lock.0;

	let xt = compose_extrinsic_offline!(
		signer,
		call,
		nonce_value,
		Era::Immortal,
		genesis_hash,
		genesis_hash,
		RUNTIME_SPEC_VERSION,
		RUNTIME_TRANSACTION_VERSION
	)
	.encode();

	*nonce_lock = Nonce(nonce_value + 1);
	std::mem::drop(nonce_lock);

	write_slice_and_whitespace_pad(extrinsic_slice, xt);
	sgx_status_t::SGX_SUCCESS
}

/// Call this once at worker startup to initialize the TOP pool and direct invocation RPC server.
///
/// This function will run the RPC server on the same thread as it is called and will loop there.
/// That means that this function will not return as long as the RPC server is running. The calling
/// code should therefore spawn a new thread when calling this function.
#[no_mangle]
pub unsafe extern "C" fn init_direct_invocation_server(
	server_addr: *const u8,
	server_addr_size: usize,
) -> sgx_status_t {
	let mut server_addr_encoded = slice::from_raw_parts(server_addr, server_addr_size);

	let server_addr = match String::decode(&mut server_addr_encoded) {
		Ok(s) => s,
		Err(e) => {
			error!("Decoding RPC server address failed. Error: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	let watch_extractor = Arc::new(create_determine_watch::<Hash>());
	let connection_registry = Arc::new(ConnectionRegistry::<Hash, TungsteniteWsConnection>::new());

	let io_handler = public_api_rpc_handler();
	let rpc_handler = Arc::new(RpcWsHandler::new(io_handler, watch_extractor, connection_registry));

	run_ws_server(server_addr.as_str(), rpc_handler);

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn init_light_client(
	genesis_header: *const u8,
	genesis_header_size: usize,
	authority_list: *const u8,
	authority_list_size: usize,
	authority_proof: *const u8,
	authority_proof_size: usize,
	latest_header: *mut u8,
	latest_header_size: usize,
) -> sgx_status_t {
	info!("Initializing light client!");

	let mut header = slice::from_raw_parts(genesis_header, genesis_header_size);
	let latest_header_slice = slice::from_raw_parts_mut(latest_header, latest_header_size);
	let mut auth = slice::from_raw_parts(authority_list, authority_list_size);
	let mut proof = slice::from_raw_parts(authority_proof, authority_proof_size);

	let header = match Header::decode(&mut header) {
		Ok(h) => h,
		Err(e) => {
			error!("Decoding Header failed. Error: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	let auth = match VersionedAuthorityList::decode(&mut auth) {
		Ok(a) => a,
		Err(e) => {
			error!("Decoding VersionedAuthorityList failed. Error: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	let proof = match StorageProof::decode(&mut proof) {
		Ok(h) => h,
		Err(e) => {
			error!("Decoding Header failed. Error: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	match itc_parentchain::light_client::io::read_or_init_validator::<Block>(header, auth, proof) {
		Ok(header) => write_slice_and_whitespace_pad(latest_header_slice, header.encode()),
		Err(e) => return e.into(),
	}

	// Initialize the global parentchain block import dispatcher instance.
	let signer = match Ed25519Seal::unseal() {
		Ok(s) => s,
		Err(e) => {
			error!("Error retrieving signer key pair: {:?}", e);
			return sgx_status_t::SGX_ERROR_UNEXPECTED
		},
	};

	let validator_access = Arc::new(EnclaveValidatorAccessor::default());
	let genesis_hash =
		match validator_access.execute_on_validator(|v| v.genesis_hash(v.num_relays())) {
			Ok(g) => g,
			Err(e) => {
				error!("Error retrieving genesis hash: {:?}", e);
				return sgx_status_t::SGX_ERROR_UNEXPECTED
			},
		};

	let ocall_api = Arc::new(OcallApi);
	let extrinsics_factory =
		Arc::new(ExtrinsicsFactory::new(genesis_hash, signer.clone(), GLOBAL_NONCE_CACHE.clone()));
	let parentchain_block_importer = Arc::new(ParentchainBlockImporter::new(
		validator_access,
		ocall_api.clone(),
		extrinsics_factory,
	));
	let block_import_dispatcher = Arc::new(ImmediateDispatcher::new(parentchain_block_importer));

	GLOBAL_PARENTCHAIN_IMPORT_IMMEDIATE_DISPATCHER_COMPONENT
		.initialize(block_import_dispatcher.clone());

	sgx_status_t::SGX_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn sync_parentchain(
	blocks_to_sync: *const u8,
	blocks_to_sync_size: usize,
	_nonce: *const u32,
) -> sgx_status_t {
	let blocks_to_sync = match Vec::<SignedBlock>::decode_raw(blocks_to_sync, blocks_to_sync_size) {
		Ok(blocks) => blocks,
		Err(e) => return Error::Codec(e).into(),
	};

	if let Err(e) = sync_parentchain_internal(blocks_to_sync) {
		return e.into()
	}

	sgx_status_t::SGX_SUCCESS
}

/// Internal [`sync_parentchain`] function to be able to use the handy `?` operator.
///
/// Sync parentchain blocks to the light-client:
/// * iterates over parentchain blocks and scans for relevant extrinsics
/// * validates and execute those extrinsics (containing indirect calls), mutating state
/// * sends `confirm_call` xt's of the executed unshielding calls
/// * sends `confirm_blocks` xt's for every synced parentchain block
fn sync_parentchain_internal(blocks_to_sync: Vec<SignedBlock>) -> Result<()> {
	let block_import_dispatcher = GLOBAL_PARENTCHAIN_IMPORT_IMMEDIATE_DISPATCHER_COMPONENT
		.get()
		.ok_or(Error::ComponentNotInitialized)?;

	block_import_dispatcher.dispatch_import(blocks_to_sync).map_err(|e| e.into())
}
