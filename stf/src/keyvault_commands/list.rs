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

use clap::ArgMatches;
use clap_nested::Command;
use log::*;

/// Lists urls of registered enclaves, one per line
pub fn keyvault_list_cli_command() -> Command<'static, str> {
    Command::new("list")
        .description("lists urls of registered enclaves, one per line")
        .runner(move |_args: &str, _matches: &ArgMatches<'_>| {
            command_runner()
        })
}

fn command_runner<'a>(
) -> Result<(), clap::Error> {
    debug!("entering keyvault list commands");
    /// LIST IMPLEMENATION HERE :
    Ok(())
}

/*
#[cfg(test)]
mod tests {

    use super::*;

    use crate::commands::test_utils::utils::{
        add_identifiers_app_args, create_identifier_args, create_main_account_args,
        create_market_id_args, create_order_id_args, PerformOperationMock,
    };
    use clap::{App, AppSettings};

    #[test]
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
    }
}
 */