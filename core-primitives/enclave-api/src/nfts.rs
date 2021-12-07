use crate::{error::Error, Enclave, EnclaveResult};
use frame_support::ensure;
use itp_enclave_api_ffi as ffi;
use sgx_types::sgx_status_t;

pub trait NFTs: Send + Sync + 'static {
	fn store_nft_data(&self, num: u8) -> EnclaveResult<()>;
}

impl NFTs for Enclave {
	fn store_nft_data(&self, num: u8) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let res = unsafe { ffi::store_nft_data(self.eid, &mut retval, num) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}
}
