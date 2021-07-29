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
use sgx_types::sgx_status_t;
use ternoa_primitives::{NFTId, BlockNumber, AccountId};
use ternoa_primitives::nfts::{NFTDetails, NFTData as NFTDataPrimitives, NFTSeriesId};
use codec::{Encode, Decode};
use log::*;

use crate::io as SgxIo;
use crate::constants::NFT_REGISTRY_DB;

pub type NFTData = NFTDataPrimitives<AccountId>;


pub trait NFTRegistryAuthorization {
    fn is_authorized(&self, owner: AccountId, nft_id: NFTId) -> bool;
}

#[derive(Debug)]
pub enum Error {
    SgxIoUnsealError(sgx_status_t),
    SgxIoSealError(sgx_status_t),
    DecodeError,
}

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
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

impl NFTRegistryAuthorization for NFTRegistry {
    fn is_authorized(&self, owner: AccountId, nft_id: NFTId) -> bool {

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
        let registry = NFTRegistry::unseal().unwrap_or_else( |_| {
                info!(
                    "[Enclave] NFT Registry DB not found, creating new! {}",
                    NFT_REGISTRY_DB
                );
                NFTRegistry::default()
            }
        );
        /*

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

    /// save NFT Registry into SgxFs
    pub fn seal() -> Result<()>{
        // save in SgxFs
        Ok(())
    }
    /// load NFT Registry from SgxFs
    pub fn unseal() -> Result<NFTRegistry> {
        let encoded = SgxIo::unseal(NFT_REGISTRY_DB).map_err(|e| Error::SgxIoUnsealError(e))?;
        NFTRegistry::decode(&mut encoded.as_slice()).map_err(|_| Error::DecodeError)
    }

    /// udpate sealed NFT Registry in SgxFs
    pub fn update(block_number: BlockNumber, id: NFTId, data: NFTData) ->  Result<()> {
        // update registry
        Ok(())
    }
}