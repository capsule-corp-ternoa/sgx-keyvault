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

use crate::{EnclaveValidatorAccessor, OcallApi};
use codec::{Decode, Encode};
use core::result::Result;
use itc_parentchain::light_client::{concurrent_access::ValidatorAccess, LightClientState};
use itp_nfts_storage::{NFTsStorage, NFTsStorageKeys};
use itp_primitives_cache::{GetPrimitives, GLOBAL_PRIMITIVES_CACHE};
use itp_sgx_crypto::Rsa3072Seal;
use itp_sgx_io::SealedIO;
use itp_storage_verifier::GetStorageVerified;
use itp_types::{
	AccountId, DirectRequestStatus, NFTData, RetrieveNftSecretRequest, RpcReturnValue,
	SignedRequest, StoreNftSecretRequest,
};
use jsonrpc_core::{serde_json::json, Error, IoHandler, Params, Value};
use std::{borrow::ToOwned, format, str, string::String, sync::Arc, vec::Vec};
use ternoa_sgx_nft::NftDbSeal;

fn compute_encoded_return_error(error_msg: &str) -> Vec<u8> {
	RpcReturnValue::from_error_message(error_msg).encode()
}

fn get_all_rpc_methods_string(io_handler: &IoHandler) -> String {
	let method_string = io_handler
		.iter()
		.map(|rp_tuple| rp_tuple.0.to_owned())
		.collect::<Vec<String>>()
		.join(", ");

	format!("methods: [{}]", method_string)
}

pub fn public_api_rpc_handler() -> IoHandler {
	let mut io = IoHandler::new();

	// nft_storeSecret
	let nft_store_secret_name: &str = "nft_storeSecret";
	io.add_sync_method(nft_store_secret_name, |params: Params| {
		let encoded_params = params.parse::<Vec<u8>>()?;
		let signed_req =
			SignedRequest::<StoreNftSecretRequest>::decode(&mut encoded_params.as_slice())
				.map_err(|_| Error::invalid_params("failed to decode signed_request"))?;

		let req = signed_req
			.get_request()
			.ok_or(Error::invalid_params("invalid request signature"))?;

		let owner = get_verified_nft_owner(req.nft_id)?;

		if owner != signed_req.signer.into() {
			return Err(Error::invalid_params(format!(
				"sender does not own the nft with id {}",
				&req.nft_id
			)))
		}

		let mut db = NftDbSeal::unseal().map_err(|_| Error::internal_error())?;

		db.upsert_sorted(req.nft_id, req.secret);

		NftDbSeal::seal(db).map_err(|_| Error::internal_error())?;

		Ok(Value::Null)
	});

	// nft_retrieveSecret
	let nft_retrieve_secret_name: &str = "nft_retrieveSecret";
	io.add_sync_method(nft_retrieve_secret_name, |params: Params| {
		let encoded_params = params.parse::<Vec<u8>>()?;
		let signed_req =
			SignedRequest::<RetrieveNftSecretRequest>::decode(&mut encoded_params.as_slice())
				.map_err(|_| Error::invalid_params("failed to decode signed_request"))?;

		let req = signed_req
			.get_request()
			.ok_or(Error::invalid_params("invalid request signature"))?;

		let owner = get_verified_nft_owner(req.nft_id)?;

		if owner != signed_req.signer.into() {
			return Err(Error::invalid_params(format!(
				"sender does not own the nft with id {}",
				&req.nft_id
			)))
		}

		let mut db = NftDbSeal::unseal().map_err(|_| Error::internal_error())?;

		let secret = db.get(req.nft_id).map_err(|_| {
			Error::invalid_params(format!("no secret stored for NFT with id '{}'", req.nft_id))
		})?;

		Ok(secret.into())
	});

	// author_getShieldingKey
	let rsa_pubkey_name: &str = "author_getShieldingKey";
	io.add_sync_method(rsa_pubkey_name, move |_: Params| {
		let rsa_pubkey = match Rsa3072Seal::unseal_pubkey() {
			Ok(key) => key,
			Err(status) => {
				let error_msg: String = format!("Could not get rsa pubkey due to: {}", status);
				return Ok(json!(compute_encoded_return_error(error_msg.as_str())))
			},
		};

		let rsa_pubkey_json = match serde_json::to_string(&rsa_pubkey) {
			Ok(k) => k,
			Err(x) => {
				let error_msg: String =
					format!("[Enclave] can't serialize rsa_pubkey {:?} {}", rsa_pubkey, x);
				return Ok(json!(compute_encoded_return_error(error_msg.as_str())))
			},
		};
		let json_value =
			RpcReturnValue::new(rsa_pubkey_json.encode(), false, DirectRequestStatus::Ok);
		Ok(json!(json_value.encode()))
	});

	let mu_ra_url_name: &str = "author_getMuRaUrl";
	io.add_sync_method(mu_ra_url_name, move |_: Params| {
		let url = match GLOBAL_PRIMITIVES_CACHE.get_mu_ra_url() {
			Ok(url) => url,
			Err(status) => {
				let error_msg: String = format!("Could not get mu ra url due to: {}", status);
				return Ok(json!(compute_encoded_return_error(error_msg.as_str())))
			},
		};

		let json_value = RpcReturnValue::new(url.encode(), false, DirectRequestStatus::Ok);
		Ok(json!(json_value.encode()))
	});

	let untrusted_url_name: &str = "author_getUntrustedUrl";
	io.add_sync_method(untrusted_url_name, move |_: Params| {
		let url = match GLOBAL_PRIMITIVES_CACHE.get_untrusted_worker_url() {
			Ok(url) => url,
			Err(status) => {
				let error_msg: String = format!("Could not get untrusted url due to: {}", status);
				return Ok(json!(compute_encoded_return_error(error_msg.as_str())))
			},
		};

		let json_value = RpcReturnValue::new(url.encode(), false, DirectRequestStatus::Ok);
		Ok(json!(json_value.encode()))
	});

	// chain_subscribeAllHeads
	let chain_subscribe_all_heads_name: &str = "chain_subscribeAllHeads";
	io.add_sync_method(chain_subscribe_all_heads_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// state_getMetadata
	let state_get_metadata_name: &str = "state_getMetadata";
	io.add_sync_method(state_get_metadata_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// state_getRuntimeVersion
	let state_get_runtime_version_name: &str = "state_getRuntimeVersion";
	io.add_sync_method(state_get_runtime_version_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// state_get
	let state_get_name: &str = "state_get";
	io.add_sync_method(state_get_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// system_health
	let state_health_name: &str = "system_health";
	io.add_sync_method(state_health_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// system_name
	let state_name_name: &str = "system_name";
	io.add_sync_method(state_name_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// system_version
	let state_version_name: &str = "system_version";
	io.add_sync_method(state_version_name, |_: Params| {
		let parsed = "world";
		Ok(Value::String(format!("hello, {}", parsed)))
	});

	// returns all rpcs methods
	let rpc_methods_string = get_all_rpc_methods_string(&io);
	io.add_sync_method("rpc_methods", move |_: Params| {
		Ok(Value::String(rpc_methods_string.to_owned()))
	});

	io
}

pub fn get_verified_nft_owner(nft_id: u32) -> Result<AccountId, Error> {
	// Get last header from light client
	let validator = Arc::new(EnclaveValidatorAccessor::default());
	let header = validator
		.execute_on_validator(|v| v.latest_finalized_header(v.num_relays()))
		.map_err(|e| Error::invalid_params(format!("failed to get header: {}", e)))?;

	// Get verified owner
	let ocall_api = Arc::new(OcallApi);
	let (_key, data): (Vec<u8>, Option<NFTData>) = ocall_api
		.get_storage_verified(NFTsStorage::data(nft_id), &header)
		.map_err(|_| Error::invalid_params("failed to get storage verified NFTData"))?
		.into_tuple();
	let owner = data
		.ok_or(Error::invalid_params(format!(
			"there is no nft with id {} in parentchain storage",
			&nft_id
		)))?
		.owner;
	Ok(owner)
}

#[cfg(feature = "test")]
pub mod tests {
	use super::*;
	use std::string::ToString;

	pub fn test_given_io_handler_methods_then_retrieve_all_names_as_string() {
		let mut io = IoHandler::new();
		let method_names: [&str; 4] = ["method1", "another_method", "fancy_thing", "solve_all"];

		for method_name in method_names.iter() {
			io.add_sync_method(method_name, |_: Params| Ok(Value::String("".to_string())));
		}

		let method_string = get_all_rpc_methods_string(&io);

		for method_name in method_names.iter() {
			assert!(method_string.contains(method_name));
		}
	}
}
