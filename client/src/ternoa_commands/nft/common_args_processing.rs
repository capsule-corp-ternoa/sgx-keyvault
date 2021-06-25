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

use crate::ternoa_commands::nft::common_arguments::{FILENAME_ARG_NAME, NFTID_ARG_NAME};
use clap::ArgMatches;

pub fn get_nft_id_from_matches(matches: &ArgMatches<'_>) -> u32 {
    get_u32_from_str(matches.value_of(NFTID_ARG_NAME).unwrap())
}

fn get_u32_from_str(arg: &str) -> u32 {
    arg.parse::<u32>()
        .unwrap_or_else(|_| panic!("failed to convert {} into an integer", arg))
}

// TODO: Add get_accountid function here?

#[cfg(test)]
mod tests {

    /*  use super::*;
    use crate::commands::common_args::add_order_args;
    use crate::commands::test_utils::utils::create_order_args;
    use clap::{App, AppSettings};
    use sp_application_crypto::sr25519;
    use sp_core::{sr25519 as sr25519_core, Pair};

    #[test]
    pub fn given_correct_args_then_map_to_order() {
        let order_args = create_order_args();
        let matches = create_test_app().get_matches_from(order_args);

        let main_account_key_pair = sr25519::AppPair::from_string("//test-account", None).unwrap();
        let main_account: AccountId =
            sr25519_core::Public::from(main_account_key_pair.public()).into();

        let order_mapping_result = get_order_from_matches(&matches, main_account);

        assert!(order_mapping_result.is_ok());

        let order = order_mapping_result.unwrap();
        assert_eq!(order.order_type, OrderType::MARKET);
        assert_eq!(order.side, OrderSide::BID);
        assert_eq!(order.quantity, 198475);
        assert_eq!(order.market_id.base, AssetId::POLKADEX);
        assert_eq!(order.market_id.quote, AssetId::DOT);
    }

    fn create_test_app<'a, 'b>() -> App<'a, 'b> {
        let test_app = App::new("test_account_details").setting(AppSettings::NoBinaryName);
        add_order_args(test_app)
    } */
}
