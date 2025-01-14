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

//! Interface for direct access to a workers rpc.

use crate::ws_client::WsClient;
use codec::Decode;
use itp_types::{DirectRequestStatus, RpcRequest, RpcResponse, RpcReturnValue};
use log::*;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use std::{
	sync::mpsc::{channel, Sender as MpscSender},
	thread,
	thread::JoinHandle,
};

pub use crate::error::{Error, Result};

#[derive(Clone)]
pub struct DirectClient {
	url: String,
}
pub trait DirectApi {
	/// Server connection with only one response.
	fn get(&self, request: &str) -> Result<String>;
	/// Server connection with more than one response.
	fn watch(&self, request: String, sender: MpscSender<String>) -> JoinHandle<()>;
	fn get_rsa_pubkey(&self) -> Result<Rsa3072PubKey>;
	fn get_mu_ra_url(&self) -> Result<String>;
	fn get_untrusted_worker_url(&self) -> Result<String>;
}

impl DirectClient {
	pub fn new(url: String) -> Self {
		Self { url }
	}
}

impl DirectApi for DirectClient {
	fn get(&self, request: &str) -> Result<String> {
		let (port_in, port_out) = channel();

		info!("[WorkerApi Direct]: (get) Sending request: {:?}", request);
		WsClient::connect(&self.url, request, &port_in, false)?;
		port_out.recv().map_err(Error::MspcReceiver)
	}

	fn watch(&self, request: String, sender: MpscSender<String>) -> JoinHandle<()> {
		info!("[WorkerApi Direct]: (watch) Sending request: {:?}", request);
		let url = self.url.clone();

		// Unwrap is fine here, because JoinHandle can be used to handle a Thread panic.
		thread::spawn(move || WsClient::connect(&url, &request, &sender, true).unwrap())
	}

	fn get_rsa_pubkey(&self) -> Result<Rsa3072PubKey> {
		let jsonrpc_call: String = RpcRequest::compose_jsonrpc_call(
			"author_getShieldingKey".to_string(),
			Vec::<u8>::new(),
		);

		// Send json rpc call to ws server.
		let response_str = Self::get(self, &jsonrpc_call)?;

		let shielding_pubkey_string = decode_from_rpc_response(&response_str)?;
		let shielding_pubkey: Rsa3072PubKey = serde_json::from_str(&shielding_pubkey_string)?;

		info!("[+] Got RSA public key of enclave");
		Ok(shielding_pubkey)
	}

	fn get_mu_ra_url(&self) -> Result<String> {
		let jsonrpc_call: String =
			RpcRequest::compose_jsonrpc_call("author_getMuRaUrl".to_string(), Vec::<u8>::new());

		// Send json rpc call to ws server.
		let response_str = Self::get(self, &jsonrpc_call)?;

		let mu_ra_url: String = decode_from_rpc_response(&response_str)?;

		info!("[+] Got mutual remote attestation url of enclave: {}", mu_ra_url);
		Ok(mu_ra_url)
	}

	fn get_untrusted_worker_url(&self) -> Result<String> {
		let jsonrpc_call: String = RpcRequest::compose_jsonrpc_call(
			"author_getUntrustedUrl".to_string(),
			Vec::<u8>::new(),
		);

		// Send json rpc call to ws server.
		let response_str = Self::get(self, &jsonrpc_call)?;

		let untrusted_url: String = decode_from_rpc_response(&response_str)?;

		info!("[+] Got untrusted websocket url of worker: {}", untrusted_url);
		Ok(untrusted_url)
	}
}

fn decode_from_rpc_response(json_rpc_response: &str) -> Result<String> {
	let rpc_response: RpcResponse<Vec<u8>> = serde_json::from_str(json_rpc_response)?;
	let rpc_return_value = RpcReturnValue::decode(&mut rpc_response.result.as_slice())?;
	let response_message = String::decode(&mut rpc_return_value.value.as_slice())?;
	match rpc_return_value.status {
		DirectRequestStatus::Ok => Ok(response_message),
		_ => Err(Error::Status(response_message)),
	}
}
