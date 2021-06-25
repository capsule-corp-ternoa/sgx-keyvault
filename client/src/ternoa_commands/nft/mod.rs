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

// example command: ./ternoa-client -p 9994 -P 2094 nft mutate AFJKA 8 FILE

use clap::AppSettings;
use clap_nested::{Commander, MultiCommand};

pub mod create;
pub mod transfer;
pub mod mutate;
pub mod common_arguments;
pub mod common_args_processing;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn nft_cmd() -> MultiCommand<'static, str, str> {
    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .name("ternoa-client")
                .version(VERSION)
                .author("Supercomputing Systems AG <info@scs.ch>")
                .about("nft calls to ternoa chain")
        })
        .add_cmd(create::nft_create_cli_command())
        .add_cmd(transfer::nft_transfer_cli_command())
        .add_cmd(mutate::nft_mutate_cli_command())
    .into_cmd("nft")
}