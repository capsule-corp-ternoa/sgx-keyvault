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

use crate::rpc::ternoa_rpc_gateway::RpcGateway;
use my_node_primitives::{AccountId, NFTId};
use substratee_stf::ShamirShare;

/// Mock implementation to be used in unit testing
pub struct RpcGatewayMock {}

impl RpcGateway for RpcGatewayMock {
    fn keyvault_get(&self, _owner: AccountId, _nft_id: NFTId) -> Option<ShamirShare> {
        Some(vec![])
    }

    fn keyvault_check(&self, _owner: AccountId, _nft_id: NFTId) -> bool {
        true
    }

    fn keyvault_provision(
        &self,
        _owner: AccountId,
        _nft_id: NFTId,
        _share: ShamirShare,
    ) -> Result<(), String> {
        Ok(())
    }
}
