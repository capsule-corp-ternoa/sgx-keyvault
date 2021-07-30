// This file is part of Polkadex.

// Copyright (C) 2020-2021 Polkadex oü and Supercomputing Systems AG
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

pub extern crate alloc;
use alloc::{boxed::Box, string::String, string::ToString, vec::Vec};

use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rpc::ternoa_rpc_gateway::RpcGateway;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use my_node_primitives::NFTId;
use substratee_node_primitives::{NFTData, Request};
use substratee_stf::{Getter, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

type RpcReturnResult<T> = std::result::Result<(T, bool, DirectRequestStatus), String>;
pub struct RpcGetNftRegistry {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcGetNftRegistry {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcGetNftRegistry {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(&self, request: Request) -> RpcReturnResult<Vec<(NFTId, NFTData)>> {
        debug!("entering keyvault_get_nft_registry RPC");

        let verified_trusted_operation = self
            .top_extractor
            .decrypt_and_verify_trusted_operation(request)?;

        match verified_trusted_operation {
            TrustedOperation::get(Getter::trusted(tgs)) => tgs,
            _ => return Err(RpcCallStatus::operation_type_mismatch.to_string()),
        };

        let registry = self.rpc_gateway.keyvault_get_nft_registry();
        debug!("Received registry: {:?}", registry);

        Ok((registry, false, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcGetNftRegistry {
    fn name() -> String {
        "keyvault_get_nft_registry".to_string()
    }
}

impl RpcMethodSync for RpcGetNftRegistry {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: Request| self.method_impl(r))
    }
}

pub mod tests {
    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call,
    };
    use crate::rpc::mocks::{
        rpc_gateway_mock::RpcGatewayMock,
        trusted_operation_extractor_mock::TrustedOperationExtractorMock,
    };
    use sp_core::Pair;
    use substratee_stf::AccountId;
    use substratee_stf::{Getter, KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
    use substratee_worker_primitives::DirectRequestStatus;

    pub fn test_given_valid_top_returns_ok() {
        // given
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(keyvault_get_nft_getter_operation()),
        });
        let rpc_gateway = Box::new(RpcGatewayMock {});

        let request = create_dummy_request();
        let rpc_keyvault_get_nft_registry = RpcGetNftRegistry::new(top_extractor, rpc_gateway);

        // when
        let result = rpc_keyvault_get_nft_registry.method_impl(request).unwrap();

        // then
        assert_eq!(result.2, DirectRequestStatus::Ok);
        assert!(!result.1); // do_watch is false
    }

    pub fn test_given_wrong_top_returns_error() {
        // given
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(keyvault_provision_operation()),
        });
        let rpc_gateway = Box::new(RpcGatewayMock {});

        let request = create_dummy_request();
        let rpc_keyvault_get_nft_registry = RpcGetNftRegistry::new(top_extractor, rpc_gateway);

        // when
        let result = rpc_keyvault_get_nft_registry.method_impl(request);

        // then
        assert!(result.is_err());
    }

    pub fn test_given_wrong_getter_returns_error() {
        // given
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(keyvault_nonce_getter_operation()),
        });
        let rpc_gateway = Box::new(RpcGatewayMock {});

        let request = create_dummy_request();
        let rpc_keyvault_get_nft_registry = RpcGetNftRegistry::new(top_extractor, rpc_gateway);

        // when
        let result = rpc_keyvault_get_nft_registry.method_impl(request);

        // then
        assert!(result.is_err());
    }

    fn keyvault_get_nft_getter_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();

        let trusted_getter = TrustedGetter::keyvault_get_nft_registry(key_pair.public().into());
        let trusted_getter_signed = trusted_getter.sign(&KeyPair::Ed25519(key_pair));

        TrustedOperation::get(Getter::trusted(trusted_getter_signed))
    }

    fn keyvault_nonce_getter_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();

        let trusted_getter = TrustedGetter::nonce(key_pair.public().into());
        let trusted_getter_signed = trusted_getter.sign(&KeyPair::Ed25519(key_pair));

        TrustedOperation::get(Getter::trusted(trusted_getter_signed))
    }

    fn keyvault_provision_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call = TrustedCall::keyvault_provision(account_id, 22, vec![]);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
