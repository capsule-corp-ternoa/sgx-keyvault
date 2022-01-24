/*
	Copyright 2021 Integritee AG and Supercomputing Systems AG
	Copyright (C) 2017-2019 Baidu, Inc. All Rights Reserved.

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

//! Type definitions for testing. Includes various mocks.

use crate::test::mocks::rpc_responder_mock::RpcResponderMock;
use itp_sgx_crypto::Aes;
use itp_test::mock::onchain_mock::OnchainMock;
use primitive_types::H256;
use sgx_crypto_helper::rsa3072::Rsa3072KeyPair;
use sp_core::ed25519 as spEd25519;

pub type TestSigner = spEd25519::Pair;

pub type TestShieldingKey = Rsa3072KeyPair;

pub type TestStateKey = Aes;

pub type TestOCallApi = OnchainMock;

pub type TestRpcResponder = RpcResponderMock<H256>;
