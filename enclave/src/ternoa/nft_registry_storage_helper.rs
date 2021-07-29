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
use my_node_primitives::{BlockNumber, NFTId};
use std::collections::HashMap;
use std::fs;
use std::vec::Vec;

use super::nft_registry::{Error, NFTData, NFTRegistry, Result};

use crate::constants::NFT_REGISTRY_DB;
use crate::io as SgxIo;

/// helper struct to encode / decode hashmap
/// and finally store it in SgxFs
#[derive(Debug, Encode, Decode)]
pub struct NFTRegistryStorageHelper {
    registry: Vec<(NFTId, NFTData)>,
    block_number: BlockNumber,
}

impl NFTRegistryStorageHelper {
    fn create_from_registry(hashmap_registry: &NFTRegistry) -> Self {
        let vec_registry: Vec<(NFTId, NFTData)> =
            hashmap_registry.registry.clone().into_iter().collect();
        NFTRegistryStorageHelper {
            block_number: hashmap_registry.block_number,
            registry: vec_registry,
        }
    }

    fn recover_registry(&self) -> NFTRegistry {
        let mut recovered_map: HashMap<NFTId, NFTData> = HashMap::new();
        let mut ids: Vec<NFTId> = Vec::new();
        for data_point in self.registry.clone() {
            recovered_map.insert(data_point.0, data_point.1);
            ids.push(data_point.0);
        }
        NFTRegistry {
            block_number: self.block_number,
            registry: recovered_map,
            nft_ids: ids,
        }
    }

    /// save NFT Registry into SgxFs
    pub fn seal(hashmap_registry: &NFTRegistry) -> Result<()> {
        debug!("backup registry state");
        if fs::copy(NFT_REGISTRY_DB, format!("{}.1", NFT_REGISTRY_DB)).is_err() {
            warn!("could not backup previous registry state");
        };
        debug!(
            "Seal Nft Registry State. Current state: {:?}",
            hashmap_registry
        );
        let helper = NFTRegistryStorageHelper::create_from_registry(hashmap_registry);
        match SgxIo::seal(helper.encode().as_slice(), NFT_REGISTRY_DB) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::SgxIoSealError(e)),
        }
    }

    /// load NFT Registry from SgxFs
    pub fn unseal() -> Result<NFTRegistry> {
        let encoded = SgxIo::unseal(NFT_REGISTRY_DB).map_err(Error::SgxIoUnsealError)?;
        let registry_codec = NFTRegistryStorageHelper::decode(&mut encoded.as_slice())
            .map_err(|_| Error::DecodeError)?;
        Ok(registry_codec.recover_registry())
    }
}
