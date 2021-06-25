/*
    Copyright 2019 Supercomputing Systems AG

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

use crate::ternoa_commands::nft::common_arguments::{add_account_id_arg, add_filename_arg};
use clap::{App, ArgMatches};
use clap_nested::Command;
use log::*;

const OWNER: &str = "owner";

/// Create a new NFT with the provided details. An ID will be auto
/// generated and logged as an event, The caller of this function
/// will become the owner of the new NFT.
/// INPUT:  AccountId (owner)
///         ASCII encoded URI to fetch additional metadata.
pub fn nft_create_cli_command() -> Command<'static, str> {
    Command::new("create")
        .description("Create a new NFT with the provided filename.")
        .options(add_arguments)
        .runner(move |_args: &str, matches: &ArgMatches<'_>| command_runner(matches))
}

fn add_arguments<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    let app_with_owner = add_account_id_arg(app, OWNER);
    add_filename_arg(app_with_owner)
}

fn command_runner<'a>(matches: &ArgMatches<'_>) -> Result<(), clap::Error> {
    let owner_ss58: &str = matches.value_of(OWNER).unwrap();
    let filename: &str = matches.value_of("filename").unwrap();
    debug!(
        "entering nft create function, owner: {}, filename: {}",
        owner_ss58, filename
    );
    // NFT CREATE FUNCTION HERE

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
