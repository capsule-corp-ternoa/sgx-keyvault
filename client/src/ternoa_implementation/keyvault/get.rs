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

use my_node_primitives::NFTId;
use substratee_stf::{TrustedOperation, TrustedGetter, KeyPair};
use crate::get_pair_from_str;
use sp_core::{sr25519 as sr25519_core, Pair};
use super::keyvault_interaction::send_direct_request_to_keyvault;
use super::constants::{SHARES_DEFAULT_PATH, SHARES_FILENAME_PREFIX};
use std::path::PathBuf;
use codec::Decode;
use sharks::Share;
use core::convert::TryFrom;
use crate::ternoa_implementation::local_storage_handler::{LocalFileStorage, VecToLinesConverter};

/// Prints all registered keyvaults and stores all url within a file (one url per line)
pub fn get(nft_id: NFTId, owner_s58: &str, url: &str, mrenclave: [u8; 32]) -> Result<(), String> {
    // Create trusted operation
    let owner =  sr25519_core::Pair::from(get_pair_from_str(owner_s58));
    let keyvault_get_top: TrustedOperation = TrustedGetter::keyvault_get(
        owner.public().into(),
        nft_id,
    )
    .sign(&KeyPair::Sr25519(owner))
    .into();
    let response_encoded = send_direct_request_to_keyvault(url, keyvault_get_top, mrenclave);
    if let Some(key_share) = Option::<Vec<u8>>::decode(&mut response_encoded.as_slice()).unwrap() {
        let filename = format!("{}_{:?}.txt", SHARES_FILENAME_PREFIX, nft_id);
        let url_handler = LocalFileStorage::new(
            PathBuf::from(SHARES_DEFAULT_PATH),
            PathBuf::from(filename),
        );
        url_handler.write(Share::try_from(key_share.as_slice()).unwrap()).unwrap();
    }
    Ok(())
}
