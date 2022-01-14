#![cfg_attr(not(feature = "std"), no_std)]

use itp_storage::{storage_map_key, StorageHasher};
use sp_std::prelude::Vec;

pub struct NFTsStorage;

// Separate the prefix from the rest because in our case we changed the storage prefix due to
// the rebranding. With the below implementation of the `NFTsStorageKeys`, we could simply
// define another struct `OtherStorage`, implement `StoragePrefix` for it, and get the
// `NFTsStorageKeys` implementation for free.
pub trait StoragePrefix {
	fn prefix() -> &'static str;
}

impl StoragePrefix for NFTsStorage {
	fn prefix() -> &'static str {
		"Nfts"
	}
}

pub trait NFTsStorageKeys {
	fn data(id: u32) -> Vec<u8>;
}

impl<S: StoragePrefix> NFTsStorageKeys for S {
	fn data(id: u32) -> Vec<u8> {
		storage_map_key(Self::prefix(), "Data", &id, &StorageHasher::Blake2_128Concat)
	}
}
