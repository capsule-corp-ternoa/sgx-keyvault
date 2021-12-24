#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
pub use sgx::*;

pub mod error;

use crate::error::{Error, Result};
use codec::{Decode, Encode};
use std::vec::Vec;

#[derive(Debug, Default, Encode, Decode, Clone)]
pub struct Nft(u32, Vec<u8>);

impl Nft {
	pub fn new(id: u32, secret: Vec<u8>) -> Self {
		Self(id, secret)
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
	pub fn upsert_sorted(&mut self, id: u32, secret: Vec<u8>) {
		match self.0.binary_search_by_key(&id, |nft| nft.0) {
			Ok(p) => self.0[p].1 = secret,
			Err(p) => self.0.insert(p, Nft::new(id, secret)),
		};
	}

	pub fn get(&mut self, id: u32) -> Result<Vec<u8>> {
		match self.0.binary_search_by_key(&id, |nft| nft.0) {
			Ok(p) => Ok(self.0[p].1.clone()),
			Err(_) => Err(Error::NftNotFound),
		}
	}
}

#[cfg(feature = "sgx")]
mod sgx {
	use super::*;
	use derive_more::Display;
	use itp_settings::files::NFT_DB;
	use itp_sgx_io::{seal, unseal, SealedIO};

	#[derive(Copy, Clone, Debug, Display)]
	pub struct NftDbSeal;

	impl SealedIO for NftDbSeal {
		type Error = Error;
		type Unsealed = NftDb;

		fn unseal() -> Result<Self::Unsealed> {
			Ok(unseal(NFT_DB)
				.map_or(Ok(NftDb::default()), |b| Decode::decode(&mut b.as_slice()))?)
		}

		fn seal(nft_db: Self::Unsealed) -> Result<()> {
			Ok(nft_db.using_encoded(|bytes| seal(bytes, NFT_DB))?)
		}
	}
}
