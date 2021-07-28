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
use alloc::{boxed::Box, string::String, string::ToString};

use crate::rpc::rpc_call_encoder::{JsonRpcCallEncoder, RpcCall, RpcCallEncoder};
use crate::rpc::rpc_info::RpcCallStatus;
use crate::rpc::ternoa_rpc_gateway::RpcGateway;
use crate::rpc::trusted_operation_verifier::TrustedOperationExtractor;
use jsonrpc_core::{BoxFuture, Params, Result as RpcResult, RpcMethodSync, Value};
use log::*;
use substratee_node_primitives::Request;
use substratee_stf::{TrustedCall, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcProvision {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcProvision {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcProvision {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(&self, request: Request) -> Result<((), bool, DirectRequestStatus), String> {
        debug!("entering keyvault_provision RPC");

        let verified_trusted_operation = self
            .top_extractor
            .decrypt_and_verify_trusted_operation(request)?;

        let trusted_call = match verified_trusted_operation {
            TrustedOperation::direct_call(tcs) => tcs.call,
            _ => return Err(RpcCallStatus::operation_type_mismatch.to_string()),
        };

        let owner = trusted_call.account().clone();

        let (nft_id, share) = match trusted_call {
            TrustedCall::keyvault_provision(_, nft_id, share) => (nft_id, share),
            _ => return Err(RpcCallStatus::operation_type_mismatch.to_string()),
        };

        match self.rpc_gateway.keyvault_provision(owner, nft_id, share) {
            Ok(()) => Ok(((), false, DirectRequestStatus::Ok)),
            Err(e) => Err(e),
        }
    }
}

impl RpcCall for RpcProvision {
    fn name() -> String {
        "keyvault_provision".to_string()
    }
}

impl RpcMethodSync for RpcProvision {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: Request| self.method_impl(r))
    }
}

pub mod tests {

    use super::*;
    use crate::rpc::mocks::dummy_builder::{
        create_dummy_account, create_dummy_request, sign_trusted_call,
    };
    use crate::rpc::mocks::rpc_gateway_mock::RpcGatewayMock;
    use crate::rpc::mocks::trusted_operation_extractor_mock::TrustedOperationExtractorMock;
    use sp_core::Pair;
    use substratee_stf::AccountId;

    pub fn test_given_valid_top_returns_ok() {
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_keyvault_provision_operation()),
        });
        let rpc_gateway = Box::new(RpcGatewayMock {});

        let request = create_dummy_request();
        let rpc_keyvault_get = RpcProvision::new(top_extractor, rpc_gateway);

        let result = rpc_keyvault_get.method_impl(request).unwrap();
        assert_eq!(result.2, DirectRequestStatus::Ok);
        assert!(!result.1); // do_watch is false
    }

    fn create_keyvault_provision_operation() -> TrustedOperation {
        let key_pair = create_dummy_account();
        let account_id: AccountId = key_pair.public().into();

        let trusted_call = TrustedCall::keyvault_provision(account_id, 22, vec![]);
        let trusted_call_signed = sign_trusted_call(trusted_call, key_pair);

        TrustedOperation::direct_call(trusted_call_signed)
    }
}
