use core::{convert::TryInto, slice};
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
pub unsafe extern "C" fn update_nft_owner(nft_id: u32, new_owner: *const u8) -> sgx_status_t {
	let new_owner: [u8; 32] = match core::slice::from_raw_parts(new_owner, 32).try_into() {
		Ok(v) => v,
		Err(_) => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
	};

	let mut db = match NftDbSeal::unseal() {
		Ok(v) => v,
		Err(e) => return e.into(),
	};

	match db.update_owner(nft_id, new_owner) {
		Ok(_) => {},
		Err(e) => return e.into(),
	}

	match NftDbSeal::seal(db) {
		Ok(_) => sgx_status_t::SGX_SUCCESS,
		Err(e) => e.into(),
	}
}

#[no_mangle]
pub unsafe extern "C" fn update_nft_secret(
	nft_id: u32,
	secret: *const u8,
	secret_size: usize,
) -> sgx_status_t {
	let secret = match slice::from_raw_parts(secret, secret_size).try_into() {
		Ok(v) => v,
		Err(_) => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
	};

	let mut db = match NftDbSeal::unseal() {
		Ok(v) => v,
		Err(e) => return e.into(),
	};

	match db.update_secret(nft_id, secret) {
		Ok(_) => {},
		Err(e) => return e.into(),
	}

	match NftDbSeal::seal(db) {
		Ok(_) => sgx_status_t::SGX_SUCCESS,
		Err(e) => e.into(),
	}
}

#[no_mangle]
pub unsafe extern "C" fn is_nft_owner(
	is_owner: *mut bool,
	nft_id: u32,
	account_id: *const u8,
) -> sgx_status_t {
	let account_id: [u8; 32] = match slice::from_raw_parts(account_id, 32).try_into() {
		Ok(v) => v,
		Err(_) => return sgx_status_t::SGX_ERROR_INVALID_PARAMETER,
	};

	let db = match NftDbSeal::unseal() {
		Ok(v) => v,
		Err(e) => return e.into(),
	};

	match db.get(nft_id) {
		Ok(data) => *is_owner = data.owner_id == account_id,
		Err(e) => return e.into(),
	}

	sgx_status_t::SGX_SUCCESS
}
