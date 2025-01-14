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

//! an RPC client to Integritee using websockets
//!
//! examples
//! integritee_cli 127.0.0.1:9944 transfer //Alice 5G9RtsTbiYJYQYMHbWfyPoeuuxNaCbC16tZ2JGrZ4gRKwz14 1000
//!
#![feature(rustc_private)]
#[macro_use]
extern crate clap;
extern crate env_logger;
extern crate log;

extern crate chrono;
use chrono::{DateTime, Utc};
use std::time::{Duration, UNIX_EPOCH};

use sp_application_crypto::{ed25519, sr25519};
use sp_keyring::AccountKeyring;
use std::path::PathBuf;

use base58::ToBase58;

use clap::{AppSettings, Arg, ArgMatches};
use clap_nested::{Command, Commander};
use codec::{Decode, Encode};
use log::*;
use my_node_runtime::{AccountId, BalancesCall, Call, Event, Hash, Signature};
use sp_core::{crypto::Ss58Codec, sr25519 as sr25519_core, Pair, H256};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	MultiSignature,
};
use std::{sync::mpsc::channel, thread};
use substrate_api_client::{
	compose_extrinsic_offline,
	rpc::{ws_client::Subscriber, WsRpcClient},
	utils::FromHexString,
	Api, GenericAddress, Metadata, RpcClient, UncheckedExtrinsicV4, XtStatus,
};

use itc_rpc_client::direct_client::{DirectApi, DirectClient as DirectWorkerApi};
use itp_api_client_extensions::{PalletNftsApi, PalletTeerexApi};
use itp_types::{
	RetrieveNftSecretRequest, RpcRequest, RpcResponse, SignableRequest, StoreNftSecretRequest,
};
use serde::{Deserialize, Serialize};
use substrate_client_keystore::{KeystoreExt, LocalKeystore};

type AccountPublic = <Signature as Verify>::Signer;
const KEYSTORE_PATH: &str = "my_keystore";
const PREFUNDING_AMOUNT: u128 = 1_000_000_000;
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
	env_logger::init();

	let res = Commander::new()
		.options(|app| {
			app.setting(AppSettings::ColoredHelp)
				.arg(
					Arg::with_name("node-url")
						.short("u")
						.long("node-url")
						.global(true)
						.takes_value(true)
						.value_name("STRING")
						.default_value("ws://127.0.0.1")
						.help("node url"),
				)
				.arg(
					Arg::with_name("node-port")
						.short("p")
						.long("node-port")
						.global(true)
						.takes_value(true)
						.value_name("STRING")
						.default_value("9944")
						.help("node port"),
				)
				.arg(
					Arg::with_name("worker-url")
						.short("U")
						.long("worker-url")
						.global(true)
						.takes_value(true)
						.value_name("STRING")
						.default_value("wss://127.0.0.1")
						.help("worker url"),
				)
				.arg(
					Arg::with_name("trusted-worker-port")
						.short("P")
						.long("trusted-worker-port")
						.global(true)
						.takes_value(true)
						.value_name("STRING")
						.default_value("2000")
						.help("worker direct invocation port"),
				)
				.name("integritee-cli")
				.version(VERSION)
				.author("Integritee AG <hello@integritee.network>")
				.about("interact with integritee-node and workers")
				.after_help("stf subcommands depend on the stf crate this has been built against")
		})
		.args(|_args, matches| matches.value_of("environment").unwrap_or("dev"))
		.add_cmd(
			Command::new("new-account")
				.description("generates a new account for the integritee chain")
				.runner(|_args: &str, _matches: &ArgMatches<'_>| {
					let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
					let key: sr25519::AppPair = store.generate().unwrap();
					drop(store);
					println!("{}", key.public().to_ss58check());
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("list-accounts")
				.description("lists all accounts in keystore for the integritee chain")
				.runner(|_args: &str, _matches: &ArgMatches<'_>| {
					let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None).unwrap();
					println!("sr25519 keys:");
					for pubkey in store.public_keys::<sr25519::AppPublic>().unwrap().into_iter() {
						println!("{}", pubkey.to_ss58check());
					}
					println!("ed25519 keys:");
					for pubkey in store.public_keys::<ed25519::AppPublic>().unwrap().into_iter() {
						println!("{}", pubkey.to_ss58check());
					}
					drop(store);
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("print-metadata")
				.description("query node metadata and print it as json to stdout")
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let meta = get_chain_api(matches).get_metadata().unwrap();
					println!("Metadata:\n {}", Metadata::pretty_format(&meta).unwrap());
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("faucet")
				.description("send some bootstrapping funds to supplied account(s)")
				.options(|app| {
					app.setting(AppSettings::ColoredHelp).arg(
						Arg::with_name("accounts")
							.takes_value(true)
							.required(true)
							.value_name("ACCOUNT")
							.multiple(true)
							.min_values(1)
							.help("Account(s) to be funded, ss58check encoded"),
					)
				})
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let api = get_chain_api(matches);
					let _api = api.set_signer(AccountKeyring::Alice.pair());
					let accounts = matches.values_of("accounts").unwrap();

					let mut nonce = _api.get_nonce().unwrap();
					for account in accounts {
						let to = get_accountid_from_str(account);
						#[allow(clippy::redundant_clone)]
						let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
							_api.clone().signer.unwrap(),
							Call::Balances(BalancesCall::transfer {
								dest: GenericAddress::Id(to.clone()),
								value: PREFUNDING_AMOUNT
							}),
							nonce,
							Era::Immortal,
							_api.genesis_hash,
							_api.genesis_hash,
							_api.runtime_version.spec_version,
							_api.runtime_version.transaction_version
						);
						// send and watch extrinsic until finalized
						println!("Faucet drips to {} (Alice's nonce={})", to, nonce);
						let _blockh =
							_api.send_extrinsic(xt.hex_encode(), XtStatus::Ready).unwrap();
						nonce += 1;
					}
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("balance")
				.description("query on-chain balance for AccountId")
				.options(|app| {
					app.setting(AppSettings::ColoredHelp).arg(
						Arg::with_name("AccountId")
							.takes_value(true)
							.required(true)
							.value_name("SS58")
							.help("AccountId in ss58check format"),
					)
				})
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let api = get_chain_api(matches);
					let account = matches.value_of("AccountId").unwrap();
					let accountid = get_accountid_from_str(account);
					let balance = if let Some(data) = api.get_account_data(&accountid).unwrap() {
						data.free
					} else {
						0
					};
					let cli_response = CliResponseFormat { status: true, result: balance };
					println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("transfer")
				.description("transfer funds from one on-chain account to another")
				.options(|app| {
					app.setting(AppSettings::ColoredHelp)
						.arg(
							Arg::with_name("from")
								.takes_value(true)
								.required(true)
								.value_name("SS58")
								.help("sender's AccountId in ss58check format"),
						)
						.arg(
							Arg::with_name("to")
								.takes_value(true)
								.required(true)
								.value_name("SS58")
								.help("recipient's AccountId in ss58check format"),
						)
						.arg(
							Arg::with_name("amount")
								.takes_value(true)
								.required(true)
								.value_name("U128")
								.help("amount to be transferred"),
						)
				})
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let api = get_chain_api(matches);
					let arg_from = matches.value_of("from").unwrap();
					let arg_to = matches.value_of("to").unwrap();
					let amount = matches
						.value_of("amount")
						.unwrap()
						.parse()
						.expect("amount can be converted to u128");
					let from = get_pair_from_str(arg_from);
					let to = get_accountid_from_str(arg_to);
					info!("from ss58 is {}", from.public().to_ss58check());
					info!("to ss58 is {}", to.to_ss58check());
					let _api = api.set_signer(sr25519_core::Pair::from(from));
					let xt = _api.balance_transfer(GenericAddress::Id(to.clone()), amount);
					let tx_hash = _api.send_extrinsic(xt.hex_encode(), XtStatus::InBlock).unwrap();
					//println!("[+] TrustedOperation got finalized. Hash: {:?}\n", tx_hash);
					//let result = _api.get_account_data(&to).unwrap().unwrap();
					//println!("balance for {} is now {}", to, result.free);
					let cli_response = CliResponseFormat { status: true, result: tx_hash };
					println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("list-workers")
				.description("query enclave registry and list all workers")
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let api = get_chain_api(matches);
					let wcount = api.enclave_count(None).unwrap();
					println!("number of workers registered: {}", wcount);
					for w in 1..=wcount {
						let enclave = api.enclave(w, None).unwrap();
						if enclave.is_none() {
							println!("error reading enclave data");
							continue
						};
						let enclave = enclave.unwrap();
						let timestamp = DateTime::<Utc>::from(
							UNIX_EPOCH + Duration::from_millis(enclave.timestamp as u64),
						);
						println!("Enclave {}", w);
						println!("   AccountId: {}", enclave.pubkey.to_ss58check());
						println!("   MRENCLAVE: {}", enclave.mr_enclave.to_base58());
						println!("   RA timestamp: {}", timestamp);
						println!("   URL: {}", enclave.url);
					}
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("nft-data")
				.description("query on chain storage to retreive a nft data")
				.options(|app| {
					app.setting(AppSettings::ColoredHelp).arg(
						Arg::with_name("nft-id")
							.takes_value(true)
							.required(true)
							.value_name("U32")
							.help("Id of the NFT"),
					)
				})
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					let arg_nft_id: u32 = matches
						.value_of("nft-id")
						.unwrap()
						.parse()
						.expect("nft-id cannot be converted to u32");
					let api = get_chain_api(matches);

					let data = match api.data(arg_nft_id) {
						Ok(v) => v,
						Err(e) => {
							println!("{}", e);
							return Ok(())
						},
					};
					//println!("data for nft with id {}: {:?}", &arg_nft_id, data);
					let cli_response = CliResponseFormat::<itp_types::NFTData> {
						status: true,
						result: data.unwrap(),
					};
					println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("listen")
				.description("listen to on-chain events")
				.options(|app| {
					app.setting(AppSettings::ColoredHelp)
						.arg(
							Arg::with_name("events")
								.short("e")
								.long("exit-after")
								.takes_value(true)
								.help("exit after given number of Integritee events"),
						)
						.arg(
							Arg::with_name("blocks")
								.short("b")
								.long("await-blocks")
								.takes_value(true)
								.help("exit after given number of blocks"),
						)
				})
				.runner(|_args: &str, matches: &ArgMatches<'_>| {
					listen(matches);
					Ok(())
				}),
		)
		.add_cmd(
			Command::new("store-nft-secret")
				.options(|app| {
					app.arg(
						Arg::with_name("account")
							.takes_value(true)
							.required(true)
							.value_name("SS58")
							.help("Sender's incognito AccountId in ss58check format"),
					)
					.arg(
						Arg::with_name("nft-id")
							.takes_value(true)
							.required(true)
							.value_name("U32")
							.help("Id of the NFT"),
					)
					.arg(
						Arg::with_name("secret")
							.takes_value(true)
							.required(true)
							.value_name("string")
							.help("Secret share to be stored"),
					)
				})
				.description("Store a NFT secret share")
				.runner(move |_args: &str, matches: &ArgMatches<'_>| {
					let arg_account = matches.value_of("account").unwrap();
					let arg_nft_id: u32 = matches
						.value_of("nft-id")
						.unwrap()
						.parse()
						.expect("nft-id cannot be converted to u32");
					let arg_secret = matches.value_of("secret").unwrap();

					let account = get_pair_from_str(arg_account);

					// compose jsonrpc call
					let rpc_method = "nft_storeSecret".to_owned();
					let data =
						StoreNftSecretRequest { nft_id: arg_nft_id, secret: arg_secret.into() }
							.sign(&sr25519_core::Pair::from(account));
					let jsonrpc_call: String =
						RpcRequest::compose_jsonrpc_call(rpc_method, data.encode());

					// call the api
					let direct_api = get_worker_api_direct(matches);
					let response_str = match direct_api.get(&jsonrpc_call) {
						Ok(resp) => resp,
						Err(_) => panic!("Error when sending direct invocation call"),
					};

					// Decode the response
					let response: RpcResponse<Option<String>> =
						match serde_json::from_str(&response_str) {
							Ok(resp) => resp,
							Err(err_msg) => panic!(
								"Error while deserialisation of the RpcResponse: {:?}",
								err_msg
							),
						};

					if let Some(error) = &response.error {
						print!("Failed to store NFT secret");
						let cli_response = CliResponseFormat::<String> {
							status: false,
							result: String::from_utf8(
								error.message.clone().unwrap_or("".to_string()).into(),
							)
							.unwrap(),
						};
						println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					} else {
						let cli_response =
							CliResponseFormat { status: true, result: "".to_string() };
						println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap())
					}

					Ok(())
				}),
		)
		.add_cmd(
			Command::new("retrieve-nft-secret")
				.options(|app| {
					app.arg(
						Arg::with_name("account")
							.takes_value(true)
							.required(true)
							.value_name("SS58")
							.help("Sender's incognito AccountId in ss58check format"),
					)
					.arg(
						Arg::with_name("nft-id")
							.takes_value(true)
							.required(true)
							.value_name("U32")
							.help("Id of the NFT"),
					)
				})
				.description("Retrieve the secret share associated with a NFT")
				.runner(move |_args: &str, matches: &ArgMatches<'_>| {
					let arg_account = matches.value_of("account").unwrap();
					let arg_nft_id: u32 = matches
						.value_of("nft-id")
						.unwrap()
						.parse()
						.expect("nft-id cannot be converted to u32");

					let account = get_pair_from_str(arg_account);

					// compose jsonrpc call
					let rpc_method = "nft_retrieveSecret".to_owned();
					let data = RetrieveNftSecretRequest { nft_id: arg_nft_id }
						.sign(&sr25519_core::Pair::from(account));
					let jsonrpc_call: String =
						RpcRequest::compose_jsonrpc_call(rpc_method, data.encode());

					// call the api
					let direct_api = get_worker_api_direct(matches);
					let response_str = match direct_api.get(&jsonrpc_call) {
						Ok(resp) => resp,
						Err(_) => panic!("Error when sending direct invocation call"),
					};

					// Decode the response
					let response: RpcResponse<Option<Vec<u8>>> =
						match serde_json::from_str(&response_str) {
							Ok(resp) => resp,
							Err(err_msg) => panic!(
								"Error while deserialisation of the RpcResponse: {:?}",
								err_msg
							),
						};

					if let Some(error) = &response.error {
						//print!("Failed to retrieve NFT secret");
						let cli_response = CliResponseFormat::<String> {
							status: false,
							result: String::from_utf8(
								error.message.clone().unwrap_or("".to_string()).into(),
							)
							.unwrap(),
						};
						println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					} else {
						let cli_response = CliResponseFormat {
							status: true,
							result: String::from_utf8(response.result.unwrap()).unwrap(),
						};
						println!("{}", CliResponseFormat::pretty_format(&cli_response).unwrap());
					}

					Ok(())
				}),
		)
		.no_cmd(|_args, _matches| {
			println!("No subcommand matched");
			Ok(())
		})
		.run();
	if let Err(e) = res {
		println!("{}", e)
	}
}

fn get_chain_api(matches: &ArgMatches<'_>) -> Api<sr25519::Pair, WsRpcClient> {
	let url = format!(
		"{}:{}",
		matches.value_of("node-url").unwrap(),
		matches.value_of("node-port").unwrap()
	);
	info!("connecting to {}", url);
	Api::<sr25519::Pair, WsRpcClient>::new(WsRpcClient::new(&url)).unwrap()
}

fn get_worker_api_direct(matches: &ArgMatches<'_>) -> DirectWorkerApi {
	let url = format!(
		"{}:{}",
		matches.value_of("worker-url").unwrap(),
		matches.value_of("trusted-worker-port").unwrap()
	);
	info!("Connecting to integritee-service-direct-port on '{}'", url);
	DirectWorkerApi::new(url)
}

#[allow(dead_code)]
#[derive(Decode)]
struct ProcessedParentchainBlockArgs {
	signer: AccountId,
	block_hash: H256,
	merkle_root: H256,
}

fn listen(matches: &ArgMatches<'_>) {
	let api = get_chain_api(matches);
	info!("Subscribing to events");
	let (events_in, events_out) = channel();
	let mut count = 0u32;
	let mut blocks = 0u32;
	api.subscribe_events(events_in).unwrap();
	loop {
		if matches.is_present("events")
			&& count >= value_t!(matches.value_of("events"), u32).unwrap()
		{
			return
		};
		if matches.is_present("blocks")
			&& blocks > value_t!(matches.value_of("blocks"), u32).unwrap()
		{
			return
		};
		let event_str = events_out.recv().unwrap();
		let _unhex = Vec::from_hex(event_str).unwrap();
		let mut _er_enc = _unhex.as_slice();
		let _events = Vec::<frame_system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
		blocks += 1;
		match _events {
			Ok(evts) =>
				for evr in &evts {
					println!("decoded: phase {:?} event {:?}", evr.phase, evr.event);
					match &evr.event {
						Event::Balances(be) => {
							println!(">>>>>>>>>> balances event: {:?}", be);
							match &be {
								pallet_balances::Event::Transfer { from, to, amount } => {
									println!("From: {:?}", from);
									println!("To: {:?}", to);
									println!("Value: {:?}", amount);
								},
								_ => {
									debug!("ignoring unsupported balances event");
								},
							}
						},
						Event::Teerex(ee) => {
							println!(">>>>>>>>>> integritee event: {:?}", ee);
							count += 1;
							match &ee {
								my_node_runtime::pallet_teerex::RawEvent::AddedEnclave(
									accountid,
									url,
								) => {
									println!(
										"AddedEnclave: {:?} at url {}",
										accountid,
										String::from_utf8(url.to_vec())
											.unwrap_or_else(|_| "error".to_string())
									);
								},
								my_node_runtime::pallet_teerex::RawEvent::RemovedEnclave(
									accountid,
								) => {
									println!("RemovedEnclave: {:?}", accountid);
								},
								my_node_runtime::pallet_teerex::RawEvent::Forwarded(shard) => {
									println!(
										"Forwarded request for shard {}",
										shard.encode().to_base58()
									);
								},
								my_node_runtime::pallet_teerex::RawEvent::ProcessedParentchainBlock(
									accountid,
									block_hash,
									merkle_root,
								) => {
									println!(
										"ProcessedParentchainBlock from {} with hash {:?} and merkle root {:?}",
										accountid, block_hash, merkle_root
									);
								},
								my_node_runtime::pallet_teerex::RawEvent::ProposedSidechainBlock(
									accountid,
									block_hash,
								) => {
									println!(
										"ProposedSidechainBlock from {} with hash {:?}",
										accountid, block_hash
									);
								},
								my_node_runtime::pallet_teerex::RawEvent::ShieldFunds(
									incognito_account,
								) => {
									println!("ShieldFunds for {:?}", incognito_account);
								},
								my_node_runtime::pallet_teerex::RawEvent::UnshieldedFunds(
									public_account,
								) => {
									println!("UnshieldFunds for {:?}", public_account);
								},
							}
						},
						_ => debug!("ignoring unsupported module event: {:?}", evr.event),
					}
				},
			Err(_) => error!("couldn't decode event record list"),
		}
	}
}

// Subscribes to the pallet_teerex events of type ProcessedParentchainBlock.
pub fn subscribe_to_processed_parentchain_block<P: Pair, Client: 'static>(
	api: Api<P, Client>,
) -> H256
where
	MultiSignature: From<P::Signature>,
	Client: RpcClient + Subscriber + Send,
{
	let (events_in, events_out) = channel();

	let _eventsubscriber = thread::Builder::new()
		.name("eventsubscriber".to_owned())
		.spawn(move || {
			api.subscribe_events(events_in.clone()).unwrap();
		})
		.unwrap();

	println!("waiting for confirmation event...");
	loop {
		let event_str = events_out.recv().unwrap();

		let _unhex = Vec::from_hex(event_str).unwrap();
		let mut _er_enc = _unhex.as_slice();
		let _events = Vec::<frame_system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
		if let Ok(evts) = _events {
			for evr in &evts {
				info!("received event {:?}", evr.event);
				if let Event::Teerex(pe) = &evr.event {
					if let my_node_runtime::pallet_teerex::RawEvent::ProcessedParentchainBlock(
						sender,
						block_hash,
						_merkle_root,
					) = &pe
					{
						println!("[+] Received processed parentchain block event from {}", sender);
						return block_hash.clone().to_owned()
					} else {
						debug!("received unknown event from Teerex: {:?}", evr.event)
					}
				}
			}
		}
	}
}

fn get_accountid_from_str(account: &str) -> AccountId {
	match &account[..2] {
		"//" => AccountPublic::from(sr25519::Pair::from_string(account, None).unwrap().public())
			.into_account(),
		_ => AccountPublic::from(sr25519::Public::from_ss58check(account).unwrap()).into_account(),
	}
}

// get a pair either form keyring (well known keys) or from the store
fn get_pair_from_str(account: &str) -> sr25519::AppPair {
	info!("getting pair for {}", account);
	match &account[..2] {
		"//" => sr25519::AppPair::from_string(account, None).unwrap(),
		_ => {
			info!("fetching from keystore at {}", &KEYSTORE_PATH);
			// open store without password protection
			let store = LocalKeystore::open(PathBuf::from(&KEYSTORE_PATH), None)
				.expect("store should exist");
			info!("store opened");
			let _pair = store
				.key_pair::<sr25519::AppPair>(
					&sr25519::Public::from_ss58check(account).unwrap().into(),
				)
				.unwrap()
				.unwrap();
			drop(store);
			_pair
		},
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CliResponseFormat<T: Serialize> {
	pub status: bool,
	pub result: T,
}

impl<T> CliResponseFormat<T>
where
	T: Serialize,
{
	pub fn pretty_format(metadata: &CliResponseFormat<T>) -> Option<String> {
		let buf = Vec::new();
		let formatter = serde_json::ser::PrettyFormatter::with_indent(b" ");
		let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
		metadata.serialize(&mut ser).unwrap();
		String::from_utf8(ser.into_inner()).ok()
	}
}