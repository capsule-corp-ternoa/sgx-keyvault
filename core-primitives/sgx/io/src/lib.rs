//! SGX file IO abstractions

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
pub use sgx::*;

/// Abstraction around IO that is supposed to use `SgxFile`. We expose it also in `std` to
/// be able to put it as trait bounds in `std` and use it in tests.
pub trait SealedIO: Sized {
	type Error: From<std::io::Error> + std::fmt::Debug + 'static;

	/// Type that is unsealed.
	type Unsealed;

	fn unseal() -> Result<Self::Unsealed, Self::Error>;
	fn seal(unsealed: Self::Unsealed) -> Result<(), Self::Error>;
}

#[cfg(feature = "sgx")]
mod sgx {
	use std::{
		io::Result,
		path::Path,
		sgxfs::{read, write},
		vec::Vec,
	};

	pub fn unseal(path: &str) -> Result<Vec<u8>> {
		read(Path::new(path))
	}

	pub fn seal(bytes: &[u8], path: &str) -> Result<()> {
		write(Path::new(path), bytes)
	}
}
