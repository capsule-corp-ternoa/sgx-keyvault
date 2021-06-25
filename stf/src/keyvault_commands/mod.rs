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


/*
pub mod cancel_order;
pub mod get_balance;
pub mod place_order;
pub mod withdraw;

mod account_details;
pub mod common_args;
pub mod common_args_processing;

mod test_utils; */

use crate::{KeyPair, TrustedCall, TrustedGetter, TrustedOperation};
use clap::{AppSettings, Arg, ArgMatches};
use clap_nested::{Command, Commander, MultiCommand};
use codec::Decode;
use log::*;
use sp_application_crypto::{ed25519, sr25519};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair};
use substrate_client_keystore::LocalKeystore;


pub mod list;
pub mod provision;
pub mod check;
pub mod get;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn keyvault_cmd<'a>(
    perform_operation: &'a dyn Fn(&ArgMatches<'_>, &TrustedOperation) -> Option<Vec<u8>>,
) -> MultiCommand<'a, str, str> {
    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .name("ternoa-client")
                .version(VERSION)
                .author("Supercomputing Systems AG <info@scs.ch>")
                .about("keyvault calls to worker enclave")
        })
        .add_cmd(list::keyvault_list_cli_command())
        .add_cmd(provision::keyvault_provision_cli_command())
        .add_cmd(check::keyvault_check_cli_command())
        .add_cmd(get::keyvault_get_cli_command())
        .into_cmd("keyvault")
}