use core::convert::TryInto;
use sgx_types::sgx_status_t;

use itp_sgx_io::SealedIO;
use ternoa_sgx_nft::{NftData, NftDbSeal};

#[no_mangle]
pub unsafe extern "C" fn store_nft_data(nft_id: u32, owner_id: *const u8) -> sgx_status_t {
	let owner_id: [u8; 32] = match core::slice::from_raw_parts(owner_id, 32).try_into() {
		Ok(v) => v,
		Err(_) => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
	};
	let data = NftData::new(owner_id);

	let mut db = match NftDbSeal::unseal() {
		Ok(v) => v,
		Err(e) => return e.into(),
	};

	match db.insert_sorted(nft_id, data) {
		Ok(_) => {},
		Err(e) => return e.into(),
	}

	match NftDbSeal::seal(db) {
		Ok(()) => sgx_status_t::SGX_SUCCESS,
		Err(e) => e.into(),
	}
}

#[no_mangle]
pub unsafe extern "C" fn update_nft_data(nft_id: u32, owner_id: *const u8) -> sgx_status_t {
	let owner_id: [u8; 32] = match core::slice::from_raw_parts(owner_id, 32).try_into() {
		Ok(v) => v,
		Err(_) => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
	};
	let data = NftData::new(owner_id);

	let mut db = match NftDbSeal::unseal() {
		Ok(v) => v,
		Err(e) => return e.into(),
	};

	match db.update(nft_id, data) {
		Ok(_) => {},
		Err(e) => return e.into(),
	}

	match NftDbSeal::seal(db) {
		Ok(_) => sgx_status_t::SGX_SUCCESS,
		Err(e) => e.into(),
	}
}
