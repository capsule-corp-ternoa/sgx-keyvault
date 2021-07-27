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

/// Prints all registered keyvaults and stores all url within a file (one url per line)
pub fn get(nft_id: NFTId, owner_s58: &str, url: &str, mrenclave: [u8; 32]) -> Result<(), String> {
    // Create trusted call signed
    let owner =  sr25519_core::Pair::from(get_pair_from_str(owner_s58));
    let get_balance_top: TrustedOperation = TrustedGetter::keyvault_get(
        owner.public().into(),
        nft_id,
    )
    .sign(&KeyPair::Sr25519(owner))
    .into();
    // TODO: save response (= shamir shard) in file storage. This function call will be implemented in issue #5
    Ok(())
}
