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

use crate::cli_utils::account_parsing::get_identifiers;
use crate::cli_utils::common_operations::get_trusted_nonce;
use crate::cli_utils::common_types::OperationRunner;
use crate::commands::account_details::AccountDetails;
use crate::commands::common_args::{
    add_main_account_args, add_order_id_args, add_proxy_account_args, add_market_id_args,
};
use crate::commands::common_args_processing::get_cancel_order_from_matches;
use crate::{KeyPair, TrustedCall, TrustedOperation};
use clap::{App, ArgMatches};
use clap_nested::Command;
use log::*;

pub fn encrypt_cli_command(perform_operation: OperationRunner) -> Command<str> {
    Command::new("encrypt")
        .description("Generates an AES256 key, encrypts and stores the input data")
        .options(add_app_args)
        .runner(move |_args: &str, matches: &ArgMatches<'_>| {
            command_runner(matches, perform_operation)
        })
}

fn add_app_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name("data")
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("data to be encrypted"),
    )
}

fn command_runner<'a>(
    matches: &ArgMatches<'_>,
    perform_operation: OperationRunner<'a>,
) -> Result<(), clap::Error> {
    let data = matches.value_of("data").unwrap();

    // ENCRYPT FUNCTION HERE #2

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    use crate::commands::test_utils::utils::{
        add_identifiers_app_args, create_identifier_args, create_main_account_args,
        create_market_id_args, create_order_id_args, PerformOperationMock,
    };
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
