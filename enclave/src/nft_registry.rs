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
use std::vec::Vec;
use std::collections::HashMap;
use ternoa_primitives::{NFTId, BlockNumber, AccountId};
//use ternoa_primitives::nfts::{NFTDetails, NFTData};
use codec::{Encode, Decode};
use log::*;

use crate::io as SgxIo;
use crate::constants::NFT_REGISTRY_DB;

/// How the NFT series id is encoded.
/// FIXME: Copy pasted from chain - maybe solvable with import?
type NFTSeriesId = u32;


/// Data related to an NFT, such as who is its owner.
/// FIXME: Copy pasted from chain - maybe solvable with import?
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
pub struct NFTData {
    pub owner: AccountId,
    pub details: NFTDetails,
    /// Set to true to prevent further modifications to the details struct
    //pub sealed: bool,
    /// Set to true to prevent changes to the owner variable
    //pub locked: bool,
}

/// Data related to NFTs on the Ternoa Chain.
/// FIXME: Copy pasted from chain - maybe solvable with import?
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, Debug)]
pub struct NFTDetails {
    /// ASCII encoded URI to fetch additional metadata.
    pub offchain_uri: Vec<u8>,
    /// The series id that this nft belongs to.
    pub series_id: NFTSeriesId,
    /// Capsule flag.
    pub is_capsule: bool,
}


pub struct NFTRegistry {
    block_number: BlockNumber,
    registry: HashMap<NFTId, NFTData>,
    nft_ids: Vec<NFTId> // optional, not sure if this is necessary
}

impl Default for NFTRegistry {
    fn default() -> Self {
        Self::new(0, HashMap::default(), vec![])
    }
}

impl NFTRegistry {
    pub fn new(block_number: BlockNumber, registry: HashMap<NFTId, NFTData>, nft_ids: Vec<NFTId>) -> Self {
        NFTRegistry {
            block_number,
            registry,
            nft_ids,
        }
    }
    /// load or create new if not in storage
    pub fn load_or_intialize() -> Self {
        if SgxIo::unseal(NFT_REGISTRY_DB).is_err() {
            info!(
                "[Enclave] NFT Registry DB not found, creating new! {}",
                NFT_REGISTRY_DB
            );
            return init_validator(header, auth, proof);
        }
        /*if SgxFile::open(CHAIN_RELAY_DB).is_err() {
            info!(
                "[Enclave] ChainRelay DB not found, creating new! {}",
                CHAIN_RELAY_DB
            );
            return init_validator(header, auth, proof);
        }

        let validator = unseal().sgx_error_with_log("Error reading validator")?;

        let genesis = validator.genesis_hash(validator.num_relays).unwrap();
        if genesis == header.hash() {
            info!(
                "Found already initialized chain relay with Genesis Hash: {:?}",
                genesis
            );
            info!("Chain Relay state: {:?}", validator);
            Ok(validator
                .latest_finalized_header(validator.num_relays)
                .unwrap())
        } else {
            init_validator(header, auth, proof)
        } */
        NFTRegistry::new(0, HashMap::new(), vec![])
    }

    pub fn seal() {
        // save in SgxFs
    }

    pub fn unseal() {
        // load from SgxFs
    }

    pub fn update(block_number: BlockNumber, id: NFTId, data: NFTData) {
        // update registry
    }
}