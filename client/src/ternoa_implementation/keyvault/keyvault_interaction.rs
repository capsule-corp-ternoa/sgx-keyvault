/*
    Copyright 2019 Supercomputing Systems AG

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

use codec::{Decode, Encode};
use log::*;
use my_node_runtime::substratee_registry::Request;
use sgx_crypto_helper::rsa3072::Rsa3072PubKey;
use std::sync::mpsc::channel;
use substratee_stf::{Getter, TrustedCall, TrustedGetter, TrustedOperation};
use substratee_worker_api::direct_client::DirectApi as DirectWorkerApi;
use substratee_worker_primitives::{DirectRequestStatus, RpcRequest, RpcResponse, RpcReturnValue};

/// sends a rpc watch request to the worker api server
pub fn send_direct_request_to_keyvault(
    url: &str,
    operation_call: TrustedOperation,
    mrenclave: [u8; 32],
) -> Result<Vec<u8>, String> {
    let keyvault = DirectWorkerApi::new(url.to_string());
    // encrypt trusted operation
    let operation_call_encrypted = match encrypt(keyvault.clone(), operation_call.clone()) {
        Ok(encrypted) => encrypted,
        Err(msg) => {
            panic!("[Error] {}", msg);
        }
    };
    // compose jsonrpc call
    let data = Request {
        shard: mrenclave.into(),
        cyphertext: operation_call_encrypted,
    };

    let rpc_method_str = match get_rpc_function_name_from_top(&operation_call) {
        Some(str) => str,
        None => {
            panic!("[Error]: This type of TrustedOperation is not supported");
        }
    };
    debug!("Got trusted operation for RPC method {}", rpc_method_str);

    let direct_invocation_call = RpcRequest {
        jsonrpc: "2.0".to_owned(),
        method: rpc_method_str,
        params: data.encode(),
        id: 1,
    };
    let jsonrpc_call: String = serde_json::to_string(&direct_invocation_call).unwrap();

    // send request and listen to response
    let (sender, receiver) = channel();
    match keyvault.watch(jsonrpc_call, sender) {
        Ok(_) => {}
        Err(_) => return Err("Error sending direct invocation call".to_string()),
    }

    loop {
        match receiver.recv() {
            Ok(response) => {
                let response: RpcResponse = serde_json::from_str(&response).unwrap();
                if let Ok(return_value) = RpcReturnValue::decode(&mut response.result.as_slice()) {
                    if !return_value.do_watch {
                        match return_value.status {
                            DirectRequestStatus::Error => {
                                match String::decode(&mut return_value.value.as_slice()) {
                                    Ok(msg) => return Err(format!("[Error] {}", msg)),
                                    Err(_) => {
                                        return Err(
                                            "Could not decode response from enclave".to_string()
                                        )
                                    }
                                }
                            }
                            DirectRequestStatus::Ok => return Ok(return_value.value),
                            _ => return Err("Unexpected RequestStatus return value".to_string()),
                        }
                    } else {
                        return Err("Unexpected watching status".to_string());
                    }
                }
            }
            Err(_) => return Err("Invalid return value".to_string()),
        };
    }
}

fn encrypt<E: Encode>(keyvault: DirectWorkerApi, to_encrypt: E) -> Result<Vec<u8>, String> {
    // request shielding key used for encryption
    let shielding_pubkey: Rsa3072PubKey = keyvault.get_rsa_pubkey()?;
    let mut encrypted: Vec<u8> = Vec::new();
    if let Err(e) = shielding_pubkey.encrypt_buffer(&to_encrypt.encode(), &mut encrypted) {
        return Err(format!("Could not retrieve shielding key: {:?}", e));
    }
    Ok(encrypted)
}

pub fn get_rpc_function_name_from_top(trusted_operation: &TrustedOperation) -> Option<String> {
    match trusted_operation {
        TrustedOperation::get(getter) => match getter {
            Getter::public(_) => None,
            Getter::trusted(tgs) => match tgs.getter {
                TrustedGetter::keyvault_check(_, _) => Some("keyvault_check".to_owned()),
                TrustedGetter::keyvault_get(_, _) => Some("keyvault_get".to_owned()),
                _ => None,
            },
        },
        TrustedOperation::indirect_call(_) => None,
        TrustedOperation::direct_call(trusted_call_signed) => match trusted_call_signed.call {
            TrustedCall::keyvault_provision(_, _, _) => Some("keyvault_provision".to_owned()),
            _ => None,
        },
    }
}
