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

use std::fs::{self, File};
use std::io::stdin;
use std::io::Write;
use std::path::Path;
use std::slice;
use std::str;
use std::sync::{
    mpsc::{channel, Sender},
    Mutex,
};
use std::collections::HashMap;
use my_node_runtime::{
    substratee_registry::ShardIdentifier, Event, Hash, Header, SignedBlock, UncheckedExtrinsic,
};

use sgx_types::*;
use codec::{Decode, Encode};
use log::*;
use sp_core::{
    crypto::{AccountId32, Ss58Codec},
    sr25519,
    storage::StorageKey,
    Pair,
    H256,
};

use substratee_worker_primitives::block::{
    SignedBlock as SignedSidechainBlock,
    SidechainBlockNumber
};

use rocksdb::{DB, WriteBatch};


/// sidechain database path
const SIDECHAIN_DB_PATH: &str = "../bin/sidechainblock_db";
/// key value of sidechain db of last block
const LAST_BLOCK_KEY: &[u8] = b"last_sidechainblock";
/// key value of the stored shards vector
const STORED_SHARDS_KEY: &[u8] = b"stored_shards";

/// DB errors.
#[derive(Debug)]
pub enum DBError {
    /// RocksDB Error
    OperationalError(rocksdb::Error),
    /// Decoding Error
    DecodeError,
}

/// Contains the blocknumber and blokhash of the
/// last sidechain block
#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug, Default)]
pub struct LastSidechainBlock {
    /// hash of the last sidechain block
    hash: H256,
    /// block number of the last sidechain block
    number: SidechainBlockNumber,
}

/// Struct used to insert newly produced sidechainblocks
/// into the database
pub struct SidechainDB {
    /// database
    pub db: DB,
    /// shards in database
    pub shards: Vec<ShardIdentifier>,
    /// map to last sidechain block of every shard
    pub last_blocks: HashMap<ShardIdentifier, LastSidechainBlock>,
}

impl SidechainDB {
    pub fn new() -> Result<SidechainDB, DBError> {
        let db = DB::open_default(SIDECHAIN_DB_PATH).unwrap();
        // get shards in db
        let shards: Vec<ShardIdentifier> = match db.get(STORED_SHARDS_KEY) {
            Ok(Some(shards)) => {
                Decode::decode(&mut shards.as_slice()).unwrap()
            },
            Ok(None) => vec![],
            Err(e) => {
                error!("Could not read shards from db: {}", e);
                return Err(DBError::OperationalError(e));
            },
        };
        // get last block of each shard
        let mut last_blocks = HashMap::new();
        for shard in shards.iter() {
            match db.get((LAST_BLOCK_KEY, shard).encode()) {
                Ok(Some(last_block_encoded)) => {
                    match LastSidechainBlock::decode(&mut last_block_encoded.as_slice()) {
                        Ok(last_block) => {
                            last_blocks.insert(shard.clone(), last_block);
                        },
                        Err(e) => {
                            error!("Could not decode signed block: {:?}", e);
                            return Err(DBError::DecodeError)
                        }
                    }
                },
                Ok(None) => { },
                Err(e) => {
                    error!("Could not read shards from db: {}", e);
                    return Err(DBError::OperationalError(e));
                }
            }
        }
        Ok(SidechainDB {
            db,
            shards,
            last_blocks,
        })
    }

    /// create new SidechainDB struct from encoded signed blocks
    pub fn update_db_from_encoded(&mut self, mut encoded_signed_blocks: &[u8]) -> Result<(), DBError> {
        let signed_blocks: Vec<SignedSidechainBlock> = match Decode::decode(&mut encoded_signed_blocks) {
            Ok(blocks) => blocks,
            Err(e) => {
                error!("Could not decode signed blocks: {:?}", e);
                return Err(DBError::DecodeError)
            }
        };
        self.update_db(signed_blocks)
    }

    /// update sidechain storage
    pub fn update_db(&mut self, blocks_to_store: Vec<SignedSidechainBlock>) -> Result<(), DBError> {
        println!{"Received blocks: {:?}", blocks_to_store};
        if !blocks_to_store.is_empty() {
            let mut batch = WriteBatch::default();
            let mut new_shard = false;
            for signed_block in blocks_to_store.clone().into_iter() {
                // Block hash -> Signed Block
                let block_hash = signed_block.hash();
                batch.put(&block_hash, &signed_block.encode().as_slice());
                // (Shard, Block number) -> Blockhash (for block pruning)
                let block_shard = signed_block.block().shard_id();
                let block_nr = signed_block.block().block_number();
                batch.put(&(block_shard, block_nr).encode().as_slice(), &block_hash);
                // (last_block_key, shard) -> (Blockhash, BlockNr) current blockchain state
                let current_last_block = LastSidechainBlock{
                    hash: block_hash.into(),
                    number: block_nr,
                };
                self.last_blocks.insert(block_shard, current_last_block);
                // stored_shards_key -> vec<shard>
                if !self.shards.contains(&block_shard) {
                    self.shards.push(block_shard);
                    new_shard = true;
                }
            }
            // update stored_shards_key -> vec<shard>
            if new_shard {
                batch.put(STORED_SHARDS_KEY, self.shards.encode());
            }
            if let Err(e) = self.db.write(batch) {
                error!("Could not write batch to sidechain db due to {}", e);
                return Err(DBError::OperationalError(e));
            };
            // update last blocks ((last_block_key, shard) -> (Blockhash, BlockNr))
            for (shard, last_block) in self.last_blocks.iter() {
                if let Err(e) = self.db.put((LAST_BLOCK_KEY, shard).encode(), last_block.encode()) {
                    error!("Could not write last block to sidechain db due to {}", e);
                    return Err(DBError::OperationalError(e));
                };
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::crypto::{AccountId32, Pair};
    use sp_core::{ed25519, H256, hashing};
    use substratee_worker_primitives::block::{Block, Signature};


    /* #[test]
    fn create_new_sidechain_struct_works() {
        // given
        let signed_block_one = create_signed_block(20, H256::random());
        let signed_block_two = create_signed_block(1, H256::random());

        let mut signed_block_vector: Vec<SignedSidechainBlock> = vec![];
        signed_block_vector.push(signed_block_one.clone());
        signed_block_vector.push(signed_block_two.clone());

        // encode blocks to slice [u8]
        let encoded_blocks = signed_block_vector.encode();
        let signed_blocks_slice = unsafe {
            slice::from_raw_parts(encoded_blocks.as_ptr(), encoded_blocks.len() as usize)
        };

        // when
        let sidechain_db = SidechainDB::new().unwrap();

        // then
        assert_eq!(sidechain_db.blocks_to_store[0], signed_block_one);
        assert_eq!(sidechain_db.blocks_to_store[1], signed_block_two);
    } */

    /*#[test]
    fn update_db_works() {
        // given
        let shard_one = H256::random();
        let shard_two = H256::random();
        let signed_block_one = create_signed_block(20, shard_one);
        let signed_block_two = create_signed_block(1, shard_two);

        let mut signed_block_vector: Vec<SignedSidechainBlock> = vec![];
        signed_block_vector.push(signed_block_one.clone());
        signed_block_vector.push(signed_block_two.clone());

        // encode blocks to slice [u8]
        let encoded_blocks = signed_block_vector.encode();
        let signed_blocks_slice = unsafe {
            slice::from_raw_parts(encoded_blocks.as_ptr(), encoded_blocks.len() as usize)
        };
        let sidechain_db = SidechainDB::new_from_encoded(signed_blocks_slice).unwrap();

        // when

        // then
        assert_eq!(sidechain_db.block_to_store[0], signed_block_one);
        assert_eq!(sidechain_db.block_to_store[1], signed_block_two);
    }
 */
    fn create_signed_block(block_number: u64, shard: ShardIdentifier) -> SignedSidechainBlock {
        let signer_pair = ed25519::Pair::from_string("//Alice", None).unwrap();
        let author: AccountId32 = signer_pair.public().into();
        let parent_hash = H256::random();
        let layer_one_head = H256::random();
        let signed_top_hashes = vec![];
        let encrypted_payload: Vec<u8> = vec![];

        let block = Block::construct_block(
            author,
            block_number,
            parent_hash.clone(),
            layer_one_head.clone(),
            shard.clone(),
            signed_top_hashes.clone(),
            encrypted_payload.clone(),
        );
        block.sign(&signer_pair)
    }
}