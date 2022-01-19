/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Licensed under the Apache License, Version 2.0 (the "License");
	you may not use this file except in compliance with the License.
	You may obtain a copy of the License at

		http://www.apache.org/licenses/LICENSE-2.0


	Unless required by applicable law or agreed to in writing, software
	distributed under the License is distributed on an "AS IS" BASIS,
	WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
	See the License for the specific language governing permissions and
	limitations under the License.
*/

use crate::{
	attestation,
	ocall::OcallApi,
	rpc,
	sync::tests::{enclave_rw_lock_works, sidechain_rw_lock_works},
	test::{cert_tests::*, mocks::rpc_responder_mock::RpcResponderMock},
};
use itp_ocall_api::EnclaveAttestationOCallApi;
use itp_sgx_crypto::Aes;
use itp_test::mock::{
	handle_state_mock, handle_state_mock::HandleStateMock,
	shielding_crypto_mock::ShieldingCryptoMock,
};
use itp_types::{Block, Header, MrEnclave};
use its_sidechain::{
	top_pool::{basic_pool::BasicPool, pool::ExtrinsicHash},
	top_pool_rpc_author::{
		api::SidechainApi, author::Author, author_tests, top_filter::AllowAllTopsFilter,
	},
};
use sgx_tunittest::*;
use sgx_types::size_t;
use sp_core::{crypto::Pair, ed25519 as spEd25519};
use sp_runtime::traits::Header as HeaderT;
use std::{string::String, sync::Arc, vec::Vec};

type TestRpcResponder = RpcResponderMock<ExtrinsicHash<SidechainApi<Block>>>;
type TestTopPool = BasicPool<SidechainApi<Block>, Block, TestRpcResponder>;
type TestRpcAuthor = Author<TestTopPool, AllowAllTopsFilter, HandleStateMock, ShieldingCryptoMock>;

#[no_mangle]
pub extern "C" fn test_main_entrance() -> size_t {
	rsgx_unit_tests!(
		attestation::tests::decode_spid_works,
		itp_stf_state_handler::tests::test_write_and_load_state_works,
		itp_stf_state_handler::tests::test_sgx_state_decode_encode_works,
		itp_stf_state_handler::tests::test_encrypt_decrypt_state_type_works,
		itp_stf_state_handler::tests::test_write_access_locks_read_until_finished,
		itp_stf_state_handler::tests::test_ensure_subsequent_state_loads_have_same_hash,
		// needs node to be running.. unit tests?
		// test_ocall_worker_request,
		rpc::worker_api_direct::tests::test_given_io_handler_methods_then_retrieve_all_names_as_string,
		author_tests::top_encryption_works,
		author_tests::submitting_to_author_inserts_in_pool,
		author_tests::submitting_call_to_author_when_top_is_filtered_returns_error,
		author_tests::submitting_getter_to_author_when_top_is_filtered_inserts_in_pool,
		handle_state_mock::tests::initialized_shards_list_is_empty,
		handle_state_mock::tests::shard_exists_after_inserting,
		handle_state_mock::tests::load_initialized_inserts_default_state,
		handle_state_mock::tests::load_mutate_and_write_works,
		handle_state_mock::tests::ensure_subsequent_state_loads_have_same_hash,
		handle_state_mock::tests::ensure_encode_and_encrypt_does_not_affect_state_hash,
		// mra cert tests
		test_verify_mra_cert_should_work,
		test_verify_wrong_cert_is_err,
		test_given_wrong_platform_info_when_verifying_attestation_report_then_return_error,
		// sync tests
		sidechain_rw_lock_works,
		enclave_rw_lock_works,
		// these unit test (?) need an ipfs node running..
		// ipfs::test_creates_ipfs_content_struct_works,
		// ipfs::test_verification_ok_for_correct_content,
		// ipfs::test_verification_fails_for_incorrect_content,
		// test_ocall_read_write_ipfs,
	)
}

// helper functions
pub fn test_top_pool() -> TestTopPool {
	let chain_api = Arc::new(SidechainApi::<Block>::new());
	let top_pool =
		BasicPool::create(Default::default(), chain_api, Arc::new(TestRpcResponder::new()));

	top_pool
}

pub fn state_key() -> Aes {
	Aes::default()
}

/// Returns all the things that are commonly used in tests and runs
/// `ensure_no_empty_shard_directory_exists`
pub fn test_setup() -> (Arc<TestRpcAuthor>, MrEnclave, ShieldingCryptoMock, Arc<HandleStateMock>) {
	let state_handler = Arc::new(HandleStateMock::default());
	let top_pool = test_top_pool();
	let mrenclave = OcallApi.get_mrenclave_of_self().unwrap().m;

	let encryption_key = ShieldingCryptoMock::default();
	(
		Arc::new(TestRpcAuthor::new(
			Arc::new(top_pool),
			AllowAllTopsFilter,
			state_handler.clone(),
			encryption_key.clone(),
		)),
		mrenclave,
		encryption_key,
		state_handler,
	)
}

/// Some random account that has no funds in the `Stf`'s `test_genesis` config.
pub fn unfunded_public() -> spEd25519::Public {
	spEd25519::Public::from_raw(*b"asdfasdfadsfasdfasfasdadfadfasdf")
}

pub fn test_account() -> spEd25519::Pair {
	spEd25519::Pair::from_seed(b"42315678901234567890123456789012")
}

/// Just some random onchain header
pub fn latest_parentchain_header() -> Header {
	Header::new(1, Default::default(), Default::default(), [69; 32].into(), Default::default())
}
