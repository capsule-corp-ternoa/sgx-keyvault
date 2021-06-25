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

use clap::{App, Arg};

pub const NFTID_ARG_NAME: &str = "nftid";
pub const FILENAME_ARG_NAME: &str = "filename";

pub fn add_nft_id_arg<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(NFTID_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("U32")
            .help("NFTId"),
    )
}

pub fn add_account_id_arg<'a, 'b>(app: App<'a, 'b>, name: &'a str) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(name)
            .takes_value(true)
            .required(true)
            .value_name("SS58")
            .help("AccountId in ss58check format"),
    )
}

pub fn add_filename_arg<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(FILENAME_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("new file name to be contained in the NFT"),
        )
}
