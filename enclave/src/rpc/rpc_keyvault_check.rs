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
use substratee_stf::{Getter, TrustedGetter, TrustedOperation};
use substratee_worker_primitives::DirectRequestStatus;

pub struct RpcCheck {
    top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
    rpc_gateway: Box<dyn RpcGateway + 'static>,
}

impl RpcCheck {
    pub fn new(
        top_extractor: Box<dyn TrustedOperationExtractor + 'static>,
        rpc_gateway: Box<dyn RpcGateway + 'static>,
    ) -> Self {
        RpcCheck {
            top_extractor,
            rpc_gateway,
        }
    }

    fn method_impl(&self, request: Request) -> Result<(bool, bool, DirectRequestStatus), String> {
        debug!("entering keyvault_check RPC");

        let verified_trusted_operation = self
            .top_extractor
            .decrypt_and_verify_trusted_operation(request)?;

        let trusted_getter_signed = match verified_trusted_operation {
            TrustedOperation::get(Getter::trusted(tgs)) => Ok(tgs),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let owner = trusted_getter_signed.getter.account().clone();

        let nft_id = match trusted_getter_signed.getter {
            TrustedGetter::keyvault_check(_, c) => Ok(c),
            _ => Err(RpcCallStatus::operation_type_mismatch.to_string()),
        }?;

        let share_exists = match self.rpc_gateway.keyvault_check(main_account, nft_id) {
            Ok(b) => Ok(b),
            Err(e) => Err(String::from(e.as_str())),
        }?;
        debug!("Share exists: {:?}", share_exists);

        Ok((share_exists, false, DirectRequestStatus::Ok))
    }
}

impl RpcCall for RpcGetBalance {
    fn name() -> String {
        "keyvault_check".to_string()
    }
}

impl RpcMethodSync for RpcGetBalance {
    fn call(&self, params: Params) -> BoxFuture<RpcResult<Value>> {
        JsonRpcCallEncoder::call(params, &|r: Request| self.method_impl(r))
    }
}

pub mod tests {

    pub extern crate alloc;
    use crate::rpc::mocks::dummy_builder::{create_dummy_account, create_dummy_request};
    use crate::rpc::mocks::{
        rpc_gateway_mock::RpcGatewayMock,
        trusted_operation_extractor_mock::TrustedOperationExtractorMock,
    };
    use crate::rpc::rpc_keyvault_get::RpcGet;
    use alloc::boxed::Box;
    use sp_core::Pair;
    use substratee_stf::{Getter, KeyPair, TrustedGetter, TrustedOperation};
    use substratee_worker_primitives::DirectRequestStatus;

    pub fn test_given_valid_top_return_balances() {
        let top_extractor = Box::new(TrustedOperationExtractorMock {
            trusted_operation: Some(create_get_balance_getter()),
        });

        let free_balance = 500;
        let reserved_balance = 1000;
        let balances = Some(Balances {
            free: free_balance,
            reserved: reserved_balance,
        });

        let rpc_gateway = Box::new(RpcGatewayMock::mock_balances(balances, true));

        let request = create_dummy_request();
        let rpc_get_balance = RpcGetBalance::new(top_extractor, rpc_gateway);

        let result = rpc_get_balance.method_impl(request).unwrap();
        assert_eq!(result.2, DirectRequestStatus::Ok);
        assert_eq!(result.0.free, free_balance);
        assert_eq!(result.0.reserved, reserved_balance);
    }

    fn create_keyvault_check_getter() -> TrustedOperation {
        let key_pair = create_dummy_account();

        let trusted_getter = TrustedGetter::keyvault_check(key_pair.public().into(), 34);
        let trusted_getter_signed = trusted_getter.sign(&KeyPair::Ed25519(key_pair));

        TrustedOperation::get(Getter::trusted(trusted_getter_signed))
    }
}
