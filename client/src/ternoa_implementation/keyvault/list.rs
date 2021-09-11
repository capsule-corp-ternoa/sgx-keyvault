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
use base58::ToBase58;
use chrono::{DateTime, Utc};
use sp_application_crypto::sr25519;
use sp_core::crypto::Ss58Codec;
use std::time::{Duration, UNIX_EPOCH};
use substrate_api_client::Api;

use crate::get_enclave;
use crate::get_enclave_count;

use super::constants::KEYVAULT_DEFAULT_PATH;
use super::constants::KEYVAULT_DEFAULT_URLLIST_FILENAME;
use crate::ternoa_implementation::local_storage_handler::{LocalFileStorage, VecToLinesConverter};
use std::io::Result;
use std::path::PathBuf;
use substrate_api_client::rpc::WsRpcClient;

/// Prints all registered keyvaults and stores all url within a file (one url per line)
pub fn list(api: Api<sr25519::Pair, WsRpcClient>) -> Result<()> {
    let number_of_keyvaults = get_enclave_count(&api);
    println!("number of keyvaults registered: {}", number_of_keyvaults);
    let mut keyvault_urls: Vec<String> = Vec::new();
    for w in 1..=number_of_keyvaults {
        if let Some(enclave) = get_enclave(&api, w) {
            let timestamp =
                DateTime::<Utc>::from(UNIX_EPOCH + Duration::from_millis(enclave.timestamp as u64));
            let keyvault_url = String::from_utf8(enclave.url).unwrap();
            keyvault_urls.push(keyvault_url.clone());
            println!("Sgx Keyvault {}", w);
            println!("   AccountId: {}", enclave.pubkey.to_ss58check());
            println!("   MRENCLAVE: {}", enclave.mr_enclave.to_base58());
            println!("   RA timestamp: {}", timestamp);
            println!("   URL: {}", keyvault_url);
        } else {
            println!("error reading enclave data");
        };
    }

    save_urls(keyvault_urls)
}

fn save_urls(keyvault_urls: Vec<String>) -> Result<()> {
    let url_handler = LocalFileStorage::new(
        PathBuf::from(KEYVAULT_DEFAULT_PATH),
        PathBuf::from(KEYVAULT_DEFAULT_URLLIST_FILENAME),
    );
    url_handler.write_lines(keyvault_urls)
}
