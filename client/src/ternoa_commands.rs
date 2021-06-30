//  Copyright (c) 2019 Alain Brenzikofer
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//       http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use chrono::{DateTime, Utc};
use std::time::{Duration, UNIX_EPOCH};

use sgx_crypto_helper::rsa3072::Rsa3072PubKey;

use sp_application_crypto::{ed25519, sr25519};
use sp_keyring::AccountKeyring;
use std::path::PathBuf;

use base58::{FromBase58, ToBase58};

use clap::{AppSettings, Arg, ArgMatches, App};
use clap_nested::{Command, Commander, MultiCommand};
use codec::{Decode, Encode};
use log::*;
use my_node_primitives::{AccountId, Hash, Signature};
use my_node_runtime::{
    substratee_registry::{Enclave, Request},
    BalancesCall, Call, Event,
};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair, H256};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    MultiSignature,
};
use std::convert::TryFrom;
use std::result::Result as StdResult;
use std::sync::mpsc::channel;
use std::thread;
use substrate_api_client::{
    compose_extrinsic, compose_extrinsic_offline,
    events::EventsDecoder,
    extrinsic::xt_primitives::{GenericAddress, UncheckedExtrinsicV4},
    node_metadata::Metadata,
    utils::FromHexString,
    Api, XtStatus,
};

use substrate_client_keystore::LocalKeystore;
use substratee_stf::{ShardIdentifier, TrustedCallSigned, TrustedOperation};
use substratee_worker_api::direct_client::DirectApi as DirectWorkerApi;
use substratee_worker_primitives::{DirectRequestStatus, RpcRequest, RpcResponse, RpcReturnValue};


type AccountPublic = <Signature as Verify>::Signer;
use crate::VERSION;

const NFTID_ARG_NAME: &str = "nftid";
const FILENAME_ARG_NAME: &str = "filename";
const URL_ARG_NAME: &str = "url";

const OWNER: &str = "owner";
const TO: &str = "to";
const FROM: &str = "from";


/// creates an inputfile.cyphertext and inputfile.aes256 with the symmetric key and stores it locally
/// INPUT: file path as String
pub fn encrypt_cmd() -> Command<'static, str> {
    Command::new("encrypt")
        .description("Generates an AES256 key, encrypts and stores the input data")
        .options(|app| {
            app.setting(AppSettings::ColoredHelp).arg(
                Arg::with_name("filepath")
                    .takes_value(true)
                    .required(true)
                    .value_name("STRING")
                    .help("filepath of the file to be encrypted"),
            )
        })
        .runner(|_args: &str, matches: &ArgMatches<'_>| {
            let path: &str = matches.value_of("filepath").unwrap();
            debug!("entering encryption function, received filepath: {}", path);
            // ENCRYPT FUNCTION HERE #2
            Ok(())
        }
    )

}

/// decrypts cyphertext using the aes256 key stored in inputfile.aes256. for debug only
/// INPUT: file path as String
/// Optional:
/// reads key shares from second file (=keyshares file), shamir-combines the shares
/// into the original assuming the exact number of shares given that is needed
/// INPUT:  file path to decrypt as String
///         shamir key shares file path
pub fn decrypt_cmd() -> Command<'static, str> {
    Command::new("decrypt")
        .description("decrypts the entered file with stored inputfile.aes256 key")
        .options(|app| {
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
                    .required(false)
                    .value_name("STRING")
                    .help("filepath of the file containing the key shares"),
            )
        })
        .runner(|_args: &str, matches: &ArgMatches<'_>| {
            let path: &str = matches.value_of("filepath").unwrap();
            let keysharesfile = match matches.value_of("keysharesfile") {
                Some(keysharesfile) => {
                    debug!(
                        "entering decrypt shamir function, received filepaths: {},{}",
                        path, keysharesfile
                    );
                },
                None => {
                    debug!("entering decrypt function, received filepath: {}", path);
                }
            };
            Ok(())
        }
    )
}

/// Adds all nft commands
pub fn nft_commands() -> MultiCommand<'static, str, str> {
    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .name("ternoa-client")
                .version(VERSION)
                .author("Supercomputing Systems AG <info@scs.ch>")
                .about("nft calls to ternoa chain")
        })
        .add_cmd(
            Command::new("create")
                .description("Create a new NFT with the provided filename.")
                .options(|app| {
                    let app_with_owner = add_account_id_arg(app, OWNER);
                    add_filename_arg(app_with_owner)
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // Create a new NFT with the provided details. An ID will be auto
                    // generated and logged as an event, The caller of this function
                    // will become the owner of the new NFT.
                    // INPUT:  AccountId (owner)
                    //         ASCII encoded URI to fetch additional metadata.
                    let owner_ss58: &str = matches.value_of(OWNER).unwrap();
                    let filename: &str = matches.value_of("filename").unwrap();
                    debug!(
                        "entering nft create function, owner: {}, filename: {}",
                        owner_ss58, filename
                    );
                    // NFT CREATE FUNCTION HERE

                    Ok(())
                }
            )
        )
        .add_cmd(
            Command::new("mutate")
                .description("Updates NFT to new filename")
                .options(|app| {
                    let app_with_owner = add_account_id_arg(app, OWNER);
                    let app_with_nftid = add_nft_id_arg(app_with_owner);
                    add_filename_arg(app_with_nftid)
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // Update the details included in an NFT. Must be called by the owner of
                    // the NFT and while the NFT is not sealed.
                    // INPUT:  AccountId (owner)
                    //         NFTId
                    //         Filename
                    let owner_ss58: &str = matches.value_of(OWNER).unwrap();
                    let nftid = get_nft_id_from_matches(matches);
                    let filename: &str = matches.value_of("filename").unwrap();
                    debug!(
                        "entering nft mutate function, owner: {}, filename: {}, id: {:?}",
                        owner_ss58, filename, nftid
                    );
                    // NFT MUTATE FUNCTION HERE

                    Ok(())
                }
            )
        )
        .add_cmd(
            Command::new("transfer")
                .description("Create a new NFT with the provided details.")
                .options(|app| {
                    let app_with_from = add_account_id_arg(app, FROM);
                    let app_with_to = add_account_id_arg(app_with_from, TO);
                    add_nft_id_arg(app_with_to)
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // Transfer an NFT from an account to another one. Must be called by the
                    // actual owner of the NFT.
                    // INPUT:  AccountId (current owner)
                    //         AccountId (new owner)
                    //         NFTId
                    let from: &str = matches.value_of(FROM).unwrap();
                    let to: &str = matches.value_of(TO).unwrap();
                    let nftid = get_nft_id_from_matches(matches);
                    debug!(
                        "entering nft transfer function, owner: {}, new owner: {}, id: {:?}",
                        from, to, nftid
                    );
                    // TRANFERFUNCTION HERE
                    Ok(())
                }
            )
        )
        .into_cmd("nft")
}



/// Adds all keyvault commands
pub fn keyvault_commands() -> MultiCommand<'static, str, str> {
    Commander::new()
        .options(|app| {
            app.setting(AppSettings::ColoredHelp)
                .name("ternoa-client")
                .version(VERSION)
                .author("Supercomputing Systems AG <info@scs.ch>")
                .about("keyvault calls to worker enclave")
        })
        .add_cmd(
            Command::new("check")
                .description("checks if keyshare for given nftid is stored in url keyvault")
                .options(|app| {
                    let app_with_nftid = add_nft_id_arg(app);
                    add_url_arg(app_with_nftid)
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // check if the key share for NFTId is stored in the keyvault with <url>. exit code 1 if negative
                    // INPUT:  NFTId (u32)
                    //         url
                    let nftid = get_nft_id_from_matches(matches);
                    let url: &str = matches.value_of(URL_ARG_NAME).unwrap();
                    debug!(
                        "entering keyvault check function, nftid: {}, urll: {}",
                        nftid, url
                    );
                    // KEYVAULT CHECK CODE HERE

                    Ok(())
                }
            )
        )
        .add_cmd(
            Command::new("get")
                .description("returns single key share")
                .options(|app| {
                    let app_with_nftid = add_nft_id_arg(app);
                    let app_with_owner = add_account_id_arg(app_with_nftid, OWNER);
                    add_url_arg(app_with_owner)
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // returns single key share
                    // INPUT:  NFTId (u32)
                    //         owner
                    //         enclave url
                    let nftid = get_nft_id_from_matches(matches);
                    let owner_ss58: &str = matches.value_of(OWNER).unwrap();
                    let url: &str = matches.value_of(URL_ARG_NAME).unwrap();
                    debug!(
                        "entering keyvault get funtciotn, nftid: {}, owner: {}, urll: {}",
                        nftid,owner_ss58, url
                    );
                    // KEYVAULT GET CODE HERE
                    Ok(())
                }
            )
        )
        .add_cmd(
            Command::new("list")
                .description("lists urls of registered enclaves, one per line")
                .runner(|_args: &str, _matches: &ArgMatches<'_>| {
                    // Lists urls of registered enclaves, one per line
                    debug!("entering keyvault list commands");
                    // LIST IMPLEMENATION HERE :
                    Ok(())
                }
            )
        )
        .add_cmd(
            Command::new("provision")
                .description("provisions all keyvaults and verifies")
                .options(|app| {
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
                })
                .runner(|_args: &str, matches: &ArgMatches<'_>| {
                    // Will read aes256 key, shamir-split shares, provision all keyvaults and verify
                    // N: number of shares needed to recover key (must be smaller than number of urls)
                    // INPUT:  NFTId (u32)
                    //         urllist ("[...]")
                    //         N
                    let nftid = get_nft_id_from_matches(matches);
                    let urllist: &str = matches.value_of("urllist").unwrap();
                    let needed_keys: &str = matches.value_of("needed_keys").unwrap();
                    debug!(
                        "entering keyvault provision, nftid: {}, urllist: {}, N: {:?}",
                        nftid, urllist, needed_keys
                    );
                    // KEYVAULT PROVISION CODE HERE
                    Ok(())
                }
            )
        )
        .into_cmd("keyvault")
}



pub fn get_nft_id_from_matches(matches: &ArgMatches<'_>) -> u32 {
    get_u32_from_str(matches.value_of(NFTID_ARG_NAME).unwrap())
}

fn get_u32_from_str(arg: &str) -> u32 {
    arg.parse::<u32>()
        .unwrap_or_else(|_| panic!("failed to convert {} into an integer", arg))
}


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


pub fn add_url_arg<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.arg(
        Arg::with_name(URL_ARG_NAME)
            .takes_value(true)
            .required(true)
            .value_name("STRING")
            .help("url of sgx keyvault enclave"),
    )
}



/* pub fn get_rpc_function_name_from_top(trusted_operation: &TrustedOperation) -> Option<String> {
    match trusted_operation {
        TrustedOperation::get(getter) => match getter {
            public(_) => None,
            trusted(tgs) => match tgs.getter {
                TrustedGetter::get_balance(_, _, _) => Some("get_balance".to_owned()),
                _ => None,
            },
        },
        TrustedOperation::indirect_call(_) => None,
        TrustedOperation::direct_call(trusted_call_signed) => match trusted_call_signed.call {
            TrustedCall::place_order(_, _, _) => Some("place_order".to_owned()),
            TrustedCall::cancel_order(_, _, _) => Some("cancel_order".to_owned()),
            TrustedCall::withdraw(_, _, _, _) => Some("withdraw".to_owned()),
            _ => None,
        },
    }
}
 */

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