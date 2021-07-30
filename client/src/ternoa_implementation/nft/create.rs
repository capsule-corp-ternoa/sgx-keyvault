use sp_application_crypto::sr25519;
use sp_core::{sr25519 as sr25519_core, Pair};
use substrate_api_client::{compose_extrinsic, utils::FromHexString, Api, XtStatus};

use crate::{get_accountid_from_str, get_pair_from_str};
use codec::Decode;
use frame_system::Event as SystemEvent;
use log::*;
use sp_runtime::DispatchError;
use my_node_primitives::NFTId;
use my_node_runtime::Event;
use sp_core::H256 as Hash;
use std::sync::mpsc::channel;
use ternoa_pallet_nfts::Event as NFTEvent;

//TODO: import it from ternoa chain instead
//use my_node_primitives::nfts::NFTSeriesId;
pub type NFTSeriesId = u32;
pub type NFTIdOf = NFTId;

/// Create a NFT for this owner
/// The NFT contains a filename of the capsule/ciphertext file.
/// Returns the NFTid: u32
/// Note: the series id, this nft belongs to, is hardcoded to 0 (the default series id) and the capsule flag is true
pub fn create(owner_ss58: &str, filename: &str, chain_api: Api<sr25519::Pair>) -> Option<NFTId> {
    let signer = get_pair_from_str(owner_ss58);
    let chain_api = chain_api.set_signer(sr25519_core::Pair::from(signer));
    // compose the extrinsic
    let offchain_uri = filename.as_bytes().to_vec();
    let xt = compose_extrinsic!(chain_api, "Nfts", "create", offchain_uri, 0u32, true);
    let tx_hash = chain_api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    info!("nft create extrinsic sent. Block Hash: {:?}", tx_hash);
    info!("waiting for confirmation of nft create");

    //subscribe to events
    let (events_in, events_out) = channel();
    chain_api.subscribe_events(events_in).unwrap();

    let owner_account_id = get_accountid_from_str(owner_ss58);
    debug!("AccountId of signer  {:?}", owner_account_id);

    //Code to catch the created event and the errors coming from chain -> break infinite loop.
    //See issue https://github.com/scs/substrate-api-client/issues/138#issuecomment-879733584
    'outer: loop {
        let event_str = events_out.recv().unwrap();
        let _unhex = Vec::from_hex(event_str).unwrap();
        let mut _er_enc = _unhex.as_slice();
        let _events = Vec::<frame_system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
        match _events {
            Ok(evts) => {
                for evr in &evts {
                    info!("decoded: phase{:?} event {:?}", evr.phase, evr.event);
                    match &evr.event {
                        Event::ternoa_nfts(nfte) => {
                            info!("NFT event received: {:?}", nfte);
                            match &nfte {
                                NFTEvent::Created(nft_id, account_id, nft_series_id) => {
                                    info!("Created event received");
                                    debug!("NFTId: {:?}", nft_id);
                                    debug!("AccountId: {:?}", account_id);
                                    debug!("NFTSeriesId: {:?}", nft_series_id);
                                    if owner_account_id == *account_id {
                                        return Some(nft_id.to_owned() as NFTId);
                                    }
                                }
                                _ => {
                                    debug!("ignoring unsupported NFT event");
                                }
                            }
                        }
                        Event::frame_system(fse) => {
                            info!("Other frame system event received: {:?}", fse);
                            match &fse {
                                SystemEvent::ExtrinsicFailed(error, _info) => match error {
                                    DispatchError::Module { index, error, .. } => {
                                        error!(
                                                "ExtrinsicFailed Module error: module index {}, error num {}",
                                                index, error
                                            );
                                        break 'outer;
                                    }
                                    _ => debug!(
                                        "ignoring unsupported ExtrinsicFailed event. Wait ..."
                                    ),
                                },
                                _ => {
                                    debug!("ignoring unsupported frame system event");
                                }
                            }
                        }
                        _ => debug!("ignoring unsupported module event: {:?}", evr.event),
                    }
                }
            }
            Err(_) => {
                error!("couldn't decode event record list");
                break 'outer;
            }
        }
    }
    None
}
