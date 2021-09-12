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

use super::keyvault_interaction::send_direct_request_to_keyvault;
use crate::get_pair_from_str;
use codec::Decode;
use my_node_primitives::nfts::NFTId;
use sp_core::{sr25519 as sr25519_core, Pair};
use substratee_node_primitives::NFTData;
use substratee_stf::{KeyPair, TrustedGetter, TrustedOperation};

/// Prints all registered keyvaults and stores all url within a file (one url per line)
pub fn get_nft_registy(url: &str, mrenclave: [u8; 32]) -> Result<(), String> {
    // Create trusted operation
    // FIXME: for now TrustedGetter is used with default Alice account. Does this make sense?
    let signer = sr25519_core::Pair::from(get_pair_from_str("//Alice"));
    let keyvault_get_nft_registry_top: TrustedOperation =
        TrustedGetter::keyvault_get_nft_registry(signer.public().into())
            .sign(&KeyPair::Sr25519(signer))
            .into();
    let response_encoded =
        send_direct_request_to_keyvault(url, keyvault_get_nft_registry_top, mrenclave)?;
    let data = Vec::<(NFTId, NFTData)>::decode(&mut response_encoded.as_slice()).unwrap();
    for datapoint in data.iter() {
        println!("{:?} -> {:?}", datapoint.0, datapoint.1);
    }

    Ok(())
}
