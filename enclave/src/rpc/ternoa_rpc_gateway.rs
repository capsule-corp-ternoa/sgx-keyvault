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
use alloc::string::String;
use my_node_primitives::{AccountId, NFTId};
use std::sync::SgxRwLockReadGuard;
use substratee_stf::ShamirShare;

use crate::ternoa;
use ternoa::keyvault::KeyvaultStorage;
use ternoa::nft_registry::NFTRegistry;

pub type Result<T> = core::result::Result<T, String>;

/// Gateway trait from RPC API -> Ternoa gateway implementation
pub trait RpcGateway: Send + Sync {
    /// get the the shamir shard of a specifc nft id
    fn keyvault_get(&self, owner: AccountId, nft_id: NFTId) -> Result<Option<ShamirShare>>;

    /// check if the keyvault contains the shard of the given nft id
    fn keyvault_check(&self, owner: AccountId, nft_id: NFTId) -> Result<bool>;

    /// store the shamir shard of a specific nft id
    fn keyvault_provision(
        &self,
        owner: AccountId,
        nft_id: NFTId,
        share: ShamirShare,
    ) -> Result<core::result::Result<(), String>>;
}

pub struct TernoaRpcGateway {}

impl RpcGateway for TernoaRpcGateway {
    fn keyvault_get(&self, _owner: AccountId, _nft_id: NFTId) -> Result<Option<ShamirShare>> {
        // TODO: Add real function here (issue #8)
        Ok(None)
    }

    fn keyvault_check(&self, _owner: AccountId, _nft_id: NFTId) -> Result<bool> {
        // TODO: Add real function here (issue #8)
        Ok(true)
    }

    fn keyvault_provision(
        &self,
        _owner: AccountId,
        _nft_id: NFTId,
        _share: ShamirShare,
    ) -> Result<core::result::Result<(), String>> {
        let registry_lock = NFTRegistry::load().map_err(|e| format!("{}", e))?;
        let registry = registry_lock.read().map_err(|e| format!("{}", e))?;
        Ok(Ok(()))
    }
}
