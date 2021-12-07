use sgx_types::sgx_status_t;

#[no_mangle]
extern "C" fn store_nft_data(num: u8) -> sgx_status_t {
	sgx_status_t::SGX_SUCCESS
}
