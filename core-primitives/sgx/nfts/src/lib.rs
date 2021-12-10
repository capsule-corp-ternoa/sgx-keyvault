#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

pub mod error;

use codec::{Decode, Encode};
use derive_more::Display;

use itp_settings::files::NFT_DB;
use itp_sgx_io::{seal, unseal, SealedIO};
use sgx_tstd::vec::Vec;

use crate::error::{Error, Result};

#[derive(Debug, Default, Encode, Decode, Clone, Copy)]
pub struct NftData {
	pub owner_id: [u8; 32],
}

impl NftData {
	pub fn new(owner_id: [u8; 32]) -> Self {
		Self { owner_id }
	}
}

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct Nft(u32, NftData);

impl Nft {
	pub fn new(id: u32, data: NftData) -> Self {
		Self(id, data)
	}
}

#[derive(Debug, Encode, Decode)]
pub struct NftDb(Vec<Nft>);

impl Default for NftDb {
	fn default() -> Self {
		Self(Vec::new())
	}
}

impl NftDb {
	pub fn insert_sorted(&mut self, id: u32, data: NftData) -> Result<()> {
		match self.0.binary_search_by_key(&id, |nft| nft.0) {
			Ok(_) => Err(Error::NftAlreadyExist),
			Err(p) => Ok(self.0.insert(p, Nft::new(id, data))),
		}
	}

	pub fn update(&mut self, id: u32, data: NftData) -> Result<()> {
		match self.0.binary_search_by_key(&id, |nft| nft.0) {
			Ok(p) => Ok(self.0[p] = Nft::new(id, data)),
			Err(_) => Err(Error::NftNotFound),
		}
	}

	pub fn get(&self, id: u32) -> Result<NftData> {
		match self.0.binary_search_by_key(&id, |nft| nft.0) {
			Ok(p) => Ok(self.0[p].1),
			Err(_) => Err(Error::NftNotFound),
		}
	}
}

#[derive(Copy, Clone, Debug, Display)]
pub struct NftDbSeal;

impl SealedIO for NftDbSeal {
	type Error = Error;
	type Unsealed = NftDb;

	fn unseal() -> Result<Self::Unsealed> {
		Ok(unseal(NFT_DB).map_or(Ok(NftDb::default()), |b| Decode::decode(&mut b.as_slice()))?)
	}

	fn seal(unsealed: Self::Unsealed) -> Result<()> {
		Ok(unsealed.using_encoded(|bytes| seal(bytes, NFT_DB))?)
	}
}
