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

use clap::{App, Arg, ArgMatches};
use clap_nested::Command;
use log::*;

/// reads key shares from second file (=keyshares file), shamir-combines the shares
/// into the original assuming the exact number of shares given that is needed
/// INPUT:  file path to decrypt as String
///         shamir key shares file path
pub fn decrypt_shamir_cli_command() -> Command<'static, str> {
    Command::new("decrypt_shamir")
        .description("decrypts the entered file with given shamir key")
        .options(add_arguments)
        .runner(move |_args: &str, matches: &ArgMatches<'_>| command_runner(matches))
}

fn add_arguments<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name("filepath")
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("filepath of the file to be decrypted"),
    )
    .arg(
        Arg::with_name("keysharesfile")
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("filepath of the file containing the key shares"),
    )
}

fn command_runner<'a>(matches: &ArgMatches<'_>) -> Result<(), clap::Error> {
    let decrypted_file_path: &str = matches.value_of("filepath").unwrap();
    let key_shares_file_path: &str = matches.value_of("keysharesfile").unwrap();
    debug!(
        "entering decrypt shamir function, received filepaths: {},{}",
        decrypted_file_path, key_shares_file_path
    );
    // DECRYPT FUNCTION HERE

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
