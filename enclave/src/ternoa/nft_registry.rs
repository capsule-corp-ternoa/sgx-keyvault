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
use super::nft_registry_storage_helper::NFTRegistryStorageHelper;
use codec::{Decode, Encode};
use log::*;
use my_node_primitives::nfts::{NFTData as NFTDataPrimitives, NFTDetails, NFTSeriesId};
use my_node_primitives::{AccountId, BlockNumber, NFTId};
use sgx_types::sgx_status_t;
use std::collections::HashMap;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, SgxRwLock};
use std::vec::Vec;

use crate::constants::NFT_REGISTRY_DB;

pub type NFTData = NFTDataPrimitives<AccountId>;
/* pub struct NFTData<AccountId> {
    pub owner: AccountId,
    pub details: NFTDetails,
    /// Set to true to prevent further modifications to the details struct
    pub sealed: bool,
    /// Set to true to prevent changes to the owner variable
    pub locked: bool,
}
 */
// pointer to NFT Registry
static NFT_REGISTRY_MEMORY: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

pub trait NFTRegistryAuthorization {
    fn is_authorized(&self, owner: AccountId, nft_id: NFTId) -> bool;
}

#[derive(Debug)]
pub enum Error {
    SgxIoUnsealError(sgx_status_t),
    SgxIoSealError(sgx_status_t),
    CouldNotLoadFromMemory,
    DecodeError,
    InconsistentBlockNumber,
    LightValidationError,
    NFTIdOverflow,
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
    fn new(
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

    /// load registry from SgxFs into memory
    pub fn initialize() {
        // load or create registry
        let registry = NFTRegistry::unseal().unwrap_or_else(|_| {
            info!(
                "[Enclave] NFT Registry DB not found, creating new! {}",
                NFT_REGISTRY_DB
            );
            NFTRegistry::default()
        });
        // initialize pointer
        let storage_ptr = Arc::new(SgxRwLock::new(registry));
        NFT_REGISTRY_MEMORY.store(Arc::into_raw(storage_ptr) as *mut (), Ordering::SeqCst);
    }

    /// load registry from memory
    /// FIXME: Currently readers could block a write call forever if issued continuosly. One should probably
    /// introduce a functionality that ensures write lock > new read lock. Mot part of PoC
    pub fn load() -> Result<&'static SgxRwLock<Self>> {
        let ptr = NFT_REGISTRY_MEMORY.load(Ordering::SeqCst) as *mut SgxRwLock<Self>;
        if ptr.is_null() {
            error!("Could not load create order cache");
            Err(Error::CouldNotLoadFromMemory)
        } else {
            Ok(unsafe { &*ptr })
        }
    }

    /// create new nft entry
    pub fn create(&mut self, owner: AccountId, details: NFTDetails) -> Result<()> {
        let nft_id = self
            .nft_ids
            .len()
            .checked_add(1)
            .ok_or(Error::NFTIdOverflow)? as NFTId;
        let nft_data = NFTData {
            owner,
            details,
            sealed: false,
            locked: false,
        };
        self.registry.insert(nft_id, nft_data);
        self.nft_ids.push(nft_id);
        Ok(())
    }

    /// mutate nft details
    pub fn mutate(&mut self, id: NFTId, new_details: NFTDetails) {
        /* self.registry.insert(id, data);
        self.nft_ids.push(id); */
    }

    /// tranfser ownership of nft
    pub fn transfer(&mut self, id: NFTId, new_owner: AccountId) {
        /* self.registry.insert(id, data);
        self.nft_ids.push(id); */
    }

    /// uddate sealed and in memory NFT Registry in SgxFs
    pub fn update_block_number_and_seal(&mut self, block_number: BlockNumber) -> Result<()> {
        // update registry
        self.block_number = block_number;
        // seal in permanent stoage
        self.seal()
    }

    /// uddate sealed and in memory NFT Registry in SgxFs
    pub fn ensure_chain_relay_consistency(&self) -> Result<bool> {
        let validator = match crate::io::light_validation::unseal() {
            Ok(v) => v,
            Err(_) => return Err(Error::LightValidationError),
        };

        let latest_header = validator
            .latest_finalized_header(validator.num_relays)
            .map_err(|_| Error::LightValidationError)?;
        if latest_header.number == self.block_number {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// save NFT Registry into SgxFs
    fn seal(&self) -> Result<()> {
        NFTRegistryStorageHelper::seal(self)
    }
    /// load NFT Registry from SgxFs
    fn unseal() -> Result<Self> {
        NFTRegistryStorageHelper::unseal()
    }
}
