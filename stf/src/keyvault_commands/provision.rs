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

use substratee_client_primitives::common_args::{add_account_id_arg, add_filename_arg, add_nft_id_arg,};
use substratee_client_primitives::common_args_processing::get_nft_id_from_matches;

use clap::{App, ArgMatches, Arg};
use clap_nested::Command;
use log::*;

/// Will read aes256 key, shamir-split shares, provision all keyvaults and verify
/// N: number of shares needed to recover key (must be smaller than number of urls)
/// INPUT:  NFTId (u32)
///         urllist ("[...]")
///         N
pub fn keyvault_provision_cli_command() -> Command<'static, str> {
    Command::new("provision")
        .description("provisions all keyvaults and verifies")
        .options(add_arguments)
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            command_runner(matches)
        })
}

fn add_arguments<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_nftid = add_nft_id_arg(app);
    app_with_nftid.arg(
        Arg::with_name("urllist")
            .takes_value(true)
            .required(true)
            .value_name("List of Strings")
            .help("list of enclave url lists"),
    )
    .arg(
        Arg::with_name("needed_keys")
            .takes_value(true)
            .required(true)
            .value_name("u32")
            .help("specifies the minimum necessary recovery keys < #urllist"),
    )
}

fn command_runner<'a>(matches: &ArgMatches<'_>) -> Result<(), clap::Error> {
    let nftid = get_nft_id_from_matches(matches);
    let urllist: &str = matches.value_of("urllist").unwrap();
    let needed_keys: &str = matches.value_of("needed_keys").unwrap();
    debug!(
        "entering nft create function, nftid: {}, urllist: {}, N: {:?}",
        nftid, urllist, needed_keys
    );
    // KEYVAULT PROVISION CODE HERE
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use clap::{App, AppSettings};

    /*  #[test]
    fn given_the_proper_arguments_then_run_operation() {
        let args = create_cancel_order_args();
        let matches = create_test_app().get_matches_from(args);

        let perform_operation_mock = PerformOperationMock::new();

        let command_result = command_runner(
            &matches,
            &|arg_matches: &ArgMatches<'_>, top: &TrustedOperation| {
                perform_operation_mock.perform_operation_mock(arg_matches, top)
            },
        );

        assert!(command_result.is_ok());
    }

    fn create_cancel_order_args() -> Vec<String> {
        let mut main_account_arg = create_main_account_args();
        let mut order_id_args = create_order_id_args();
        let mut identifier_args = create_identifier_args();
        let mut market_id = create_market_id_args();

        main_account_arg.append(&mut order_id_args);
        main_account_arg.append(&mut identifier_args);
        main_account_arg.append(&mut market_id);


        main_account_arg
    }

    fn create_test_app<'a, 'b>() -> App<'a, 'b> {
        let test_app = App::new("test_account_details").setting(AppSettings::NoBinaryName);
        let app_with_arg = add_app_args(test_app);

        add_identifiers_app_args(app_with_arg)
    } */
}
