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
use sgx_types::sgx_status_t;
use std::collections::HashMap;
use std::vec::Vec;
use ternoa_primitives::nfts::{NFTData as NFTDataPrimitives, NFTDetails, NFTSeriesId};
use ternoa_primitives::{AccountId, BlockNumber, NFTId};
use super::nft_registry_storage_helper::NFTRegistryStorageHelper;

use crate::constants::NFT_REGISTRY_DB;
use crate::io as SgxIo;

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
    pub block_number: BlockNumber,
    pub registry: HashMap<NFTId, NFTData>,
    pub nft_ids: Vec<NFTId>, // optional, not sure if this is necessary
}


impl Default for NFTRegistry {
    fn default() -> Self {
        Self::new(0, HashMap::default(), vec![])
    }
}

impl NFTRegistryAuthorization for NFTRegistry {
    fn is_authorized(&self, owner: AccountId, nft_id: NFTId) -> bool {
        true
    }
}

impl NFTRegistry {
    pub fn new(
        block_number: BlockNumber,
        registry: HashMap<NFTId, NFTData>,
        nft_ids: Vec<NFTId>,
    ) -> Self {
        NFTRegistry {
            block_number,
            registry,
            nft_ids,
        }
    }
    /// load or create new if not in storage
    pub fn load_or_intialize() -> Self {
        let registry = NFTRegistry::unseal().unwrap_or_else(|_| {
            info!(
                "[Enclave] NFT Registry DB not found, creating new! {}",
                NFT_REGISTRY_DB
            );
            NFTRegistry::default()
        });
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
    pub fn seal(self) -> Result<()> {
        NFTRegistryStorageHelper::seal(self)
    }
    /// load NFT Registry from SgxFs
    pub fn unseal() -> Result<Self> {
       NFTRegistryStorageHelper::unseal()
    }

    /// udpate sealed NFT Registry in SgxFs
    pub fn update(block_number: BlockNumber, id: NFTId, data: NFTData) -> Result<()> {
        // update registry
        Ok(())
    }
}
