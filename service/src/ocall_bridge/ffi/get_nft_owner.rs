use crate::node_api_factory::{CreateNodeApi, GlobalUrlNodeApiFactory};
use itp_api_client_extensions::PalletNftsApi;
use sgx_types::sgx_status_t;
use std::slice;

/// # Safety
///
/// FFI are always unsafe
#[no_mangle]
pub unsafe extern "C" fn ocall_get_nft_owner(owner_id: *mut u8, nft_id: u32) -> sgx_status_t {
	get_nft_owner(owner_id, nft_id)
}

fn get_nft_owner(owner_id: *mut u8, nft_id: u32) -> sgx_status_t {
	let api = GlobalUrlNodeApiFactory.create_api();
	let owner = match api.owner(nft_id) {
		Ok(opt_d) => match opt_d {
			Some(d) => d,
			None => return sgx_status_t::SGX_ERROR_UNEXPECTED,
		},
		Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
	};

	let owner_id_slice = unsafe { slice::from_raw_parts_mut(owner_id, 32) };
	owner_id_slice.clone_from_slice(owner.as_ref());

	sgx_status_t::SGX_SUCCESS
}
