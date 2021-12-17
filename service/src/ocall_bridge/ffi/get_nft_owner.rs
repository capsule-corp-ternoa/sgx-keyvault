use crate::ocall_bridge::bridge_api::{Bridge, WorkerOnChainBridge};
use log::*;
use sgx_types::sgx_status_t;
use std::{slice, sync::Arc, vec::Vec};

/// # Safety
///
/// FFI are always unsafe
#[no_mangle]
pub unsafe extern "C" fn ocall_get_nft_owner(owner_id: *const u8, nft_id: u32) -> sgx_status_t {
	get_nft_owner(owner_id, nft_id, Bridge::get_oc_api())
}

fn get_nft_owner(
	owner_id: *const u8,
	nft_id: u32,
	oc_api: Arc<dyn WorkerOnChainBridge>,
) -> sgx_status_t {
	println!("in get_nft_owner");
	sgx_status_t::SGX_SUCCESS
}
