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
use derive_more::Display;
use log::*;
use my_node_primitives::nfts::{NFTData as NFTDataPrimitives, NFTDetails};
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

#[derive(Debug, Display)]
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
#[derive(Debug, PartialEq)]
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
        if let Some(data) = self.registry.get(&nft_id) {
            if data.owner == owner {
                return true;
            }
        }
        false
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
        debug!("entering create");
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
        debug!("entering mutate");
        if let Some(data) = self.registry.get_mut(&id) {
            data.details = new_details;
        } else {
            error!("Tried to mutate nonexistent nft id")
        }
    }

    /// tranfser ownership of nft
    pub fn transfer(&mut self, id: NFTId, new_owner: AccountId) {
        debug!("entering transfer");
        if let Some(data) = self.registry.get_mut(&id) {
            data.owner = new_owner;
        } else {
            error!("Tried to transfer nonexistent nft id")
        }
    }

    /// uddate sealed and in memory NFT Registry in SgxFs
    pub fn update_block_number_and_seal(&mut self, block_number: &BlockNumber) -> Result<()> {
        // update registry
        self.block_number = *block_number;
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
        NFTRegistryStorageHelper::seal(NFT_REGISTRY_DB, self)
    }
    /// load NFT Registry from SgxFs
    fn unseal() -> Result<Self> {
        NFTRegistryStorageHelper::unseal(NFT_REGISTRY_DB)
    }
}

pub mod test {
    use super::*;

    use my_node_primitives::nfts::NFTDetails;
    use my_node_primitives::AccountId;
    use std::fs;

    pub fn test_is_authorized_returns_true_if_registered() {
        //given
        let nft_id = 9;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let nft_data = NFTData::new(owner.clone(), details, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(nft_id, nft_data);
        let registry = NFTRegistry::new(100, hash_map, vec![]);

        // when
        let is_authorized = registry.is_authorized(owner, nft_id);

        // then
        assert!(is_authorized)
    }

    pub fn test_is_authorized_returns_false_if_nft_not_registered() {
        //given
        let nft_id = 9;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let nft_data = NFTData::new(owner.clone(), details, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(nft_id, nft_data);
        let registry = NFTRegistry::new(100, hash_map, vec![]);

        // when
        let is_authorized = registry.is_authorized(owner, 1);

        // then
        assert!(!is_authorized)
    }

    pub fn test_is_authorized_returns_false_if_wrong_owner() {
        //given
        let nft_id = 9;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let fake_owner = dummy_account_two();
        let nft_data = NFTData::new(owner, details, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(nft_id, nft_data);
        let registry = NFTRegistry::new(100, hash_map, vec![]);

        // when
        let is_authorized = registry.is_authorized(fake_owner, nft_id);

        // then
        assert!(!is_authorized)
    }

    pub fn test_initialize_and_load_pointer_works() {
        let new_block_number = 20;

        NFTRegistry::initialize();
        // get write lock
        {
            let registry_lock = NFTRegistry::load().unwrap();
            let mut write = registry_lock.write().unwrap();
            write.block_number = new_block_number;
        }
        // test if write worked
        {
            let registry_lock = NFTRegistry::load().unwrap();
            let read = registry_lock.read().unwrap();
            assert_eq!(read.block_number, new_block_number);
        }
    }

    pub fn test_create_works() {
        //given
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let mut registry = NFTRegistry::default();

        // when
        registry.create(owner.clone(), details.clone()).unwrap();

        // then
        let nft_data = registry.registry.get(&1).unwrap();
        assert_eq!(registry.nft_ids.len(), 1);
        assert_eq!(registry.nft_ids[0], 1);
        assert_eq!(nft_data.details, details);
        assert_eq!(nft_data.owner, owner);
    }

    pub fn test_mutate_works() {
        //given
        let nft_id = 7;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let new_details = NFTDetails::new(vec![0, 1, 1, 1], 1, false);
        let owner = dummy_account();
        let nft_data = NFTData::new(owner.clone(), details, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(nft_id, nft_data);
        let mut registry = NFTRegistry::new(100, hash_map, vec![]);

        // when
        registry.mutate(nft_id, new_details.clone());

        // then
        let nft_data = registry.registry.get(&nft_id).unwrap();
        assert_eq!(nft_data.details, new_details);
        assert_eq!(nft_data.owner, owner);
    }

    pub fn test_transfer_works() {
        //given
        let nft_id = 7;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let new_owner = dummy_account_two();
        let nft_data = NFTData::new(owner, details.clone(), false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(nft_id, nft_data);
        let mut registry = NFTRegistry::new(100, hash_map, vec![]);

        // when
        registry.transfer(nft_id, new_owner.clone());

        // then
        let nft_data = registry.registry.get(&nft_id).unwrap();
        assert_eq!(nft_data.details, details);
        assert_eq!(nft_data.owner, new_owner);
    }

    pub fn test_update_block_number_and_seal() {
        //given
        let block_number = 30;
        let details = NFTDetails::new(vec![10, 3, 0, 1, 2], 9, false);
        let owner = dummy_account();
        let nft_data = NFTData::new(owner, details, false, false);
        let mut hash_map: HashMap<NFTId, NFTData> = HashMap::new();
        hash_map.insert(1, nft_data);
        let mut registry = NFTRegistry::new(10, hash_map, vec![]);

        // when
        registry
            .update_block_number_and_seal(&block_number)
            .unwrap();

        // then
        let read_registry = NFTRegistry::unseal().unwrap();
        assert_eq!(read_registry.block_number, 30);

        // clean up
        fs::remove_file(NFT_REGISTRY_DB).unwrap();
        let backup_path = format!("{}.1", NFT_REGISTRY_DB);
        if fs::copy(backup_path, NFT_REGISTRY_DB).is_err() {
            warn!("could not restore previous state");
        };
    }

    fn dummy_account() -> AccountId {
        AccountId::from([
            212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133,
            88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 163, 127,
        ])
    }

    fn dummy_account_two() -> AccountId {
        AccountId::from([
            212, 53, 147, 191, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133,
            88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 163, 127,
        ])
    }
}
