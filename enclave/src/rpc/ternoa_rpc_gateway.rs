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
use super::rpc_info::RpcCallStatus;
use alloc::{string::String, string::ToString};
use log::*;
use my_node_primitives::{AccountId, NFTId};
use sgx_types::{sgx_status_t, SgxResult};
use substratee_stf::{ShamirShare, TrustedCall, TrustedOperation};

/// Gateway trait from RPC API -> Ternoa gateway implementation
pub trait RpcGateway: Send + Sync {
    /// get the the shamir shard of a specifc nft id
    fn keyvault_get(&self, owner: AccountId, nft_id: NFTId) -> Option<ShamirShare>;

    /// check if the keyvault contains the shard of the given nft id
    fn keyvault_check(&self, owner: AccountId, nft_id: NFTId) -> bool;

    /// store the shamir shard of a specific nft id
    fn keyvault_provision(
        &self,
        owner: AccountId,
        nft_id: NFTId,
        share: ShamirShare,
    ) -> Result<(), String>;
}

pub struct TernoaRpcGateway {}

impl RpcGateway for TernoaRpcGateway {
    fn keyvault_get(&self, owner: AccountId, nft_id: NFTId) -> Option<ShamirShare> {
        /* match lock_storage_and_get_balances(main_account, asset_id) {
            Ok(balance) => Ok(balance),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        } */
        None
    }

    fn keyvault_check(&self, owner: AccountId, nft_id: NFTId) -> bool {
        /* let gateway = OpenfinexPolkaDexGateway::new(OpenFinexApiImpl::new(
            OpenFinexClientInterface::new(0), // FIXME: for now hardcoded 0, but we should change that to..?
        ));
        gateway.cancel_order(main_account, proxy_acc, order) */
        true
    }

    fn keyvault_provision(
        &self,
        owner: AccountId,
        nft_id: NFTId,
        share: ShamirShare,
    ) -> Result<(), String> {
        /* match lock_storage_and_get_balances(main_account, asset_id) {
            Ok(balance) => Ok(balance),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        } */
        Ok(())
    }
}
