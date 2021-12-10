use crate::{error::Error, Enclave, EnclaveResult};
use frame_support::ensure;
use itp_enclave_api_ffi as ffi;
use sgx_types::sgx_status_t;
use sp_runtime::AccountId32;

pub trait NFTs: Send + Sync + 'static {
	fn store_nft_data(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()>;
	fn update_nft_owner(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()>;
	fn update_nft_secret(&self, nft_id: u32, secret: Vec<u8>) -> EnclaveResult<()>;
	fn is_nft_owner(&self, nft_id: u32, account_id: AccountId32) -> EnclaveResult<bool>;
}

impl NFTs for Enclave {
	fn store_nft_data(&self, nft_id: u32, owner_id: AccountId32) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let p_owner_id = AsRef::<[u8]>::as_ref(&owner_id).as_ptr();

		let res = unsafe { ffi::store_nft_data(self.eid, &mut retval, nft_id, p_owner_id) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}

	fn update_nft_owner(&self, nft_id: u32, new_owner_id: AccountId32) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let p_owner_id = AsRef::<[u8]>::as_ref(&new_owner_id).as_ptr();

		let res = unsafe { ffi::update_nft_owner(self.eid, &mut retval, nft_id, p_owner_id) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}

	fn update_nft_secret(&self, nft_id: u32, secret: Vec<u8>) -> EnclaveResult<()> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let res = unsafe {
			ffi::update_nft_secret(self.eid, &mut retval, nft_id, secret.as_ptr(), secret.len())
		};

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(())
	}

	fn is_nft_owner(&self, nft_id: u32, account_id: AccountId32) -> EnclaveResult<bool> {
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let mut is_owner = false;

		let p_owner_id = AsRef::<[u8]>::as_ref(&account_id).as_ptr();
		let p_is_owner: *mut bool = &mut is_owner;

		let res =
			unsafe { ffi::is_nft_owner(self.eid, &mut retval, p_is_owner, nft_id, p_owner_id) };

		ensure!(res == sgx_status_t::SGX_SUCCESS, Error::Sgx(res));
		ensure!(retval == sgx_status_t::SGX_SUCCESS, Error::Sgx(retval));

		EnclaveResult::Ok(is_owner)
	}
}
