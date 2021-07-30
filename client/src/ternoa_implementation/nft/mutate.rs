use crate::get_pair_from_str;
use codec::Decode;
use frame_system::Event as SystemEvent;
use log::*;
use my_node_runtime::Event;
use sp_runtime::DispatchError;
use sp_application_crypto::sr25519;
use sp_core::H256 as Hash;
use sp_core::{sr25519 as sr25519_core, Pair};
use std::sync::mpsc::channel;
use substrate_api_client::{compose_extrinsic, utils::FromHexString, Api, XtStatus};
use ternoa_pallet_nfts::Event as NFTEvent;

/// Update the file included in the NFT with id nft_id.
/// Must be called by the owner of the NFT and while the NFT is not sealed.
/// Note: the series id, this nft belongs to, is hardcoded to 0 (the default series id) and the capsule flag is true.
pub fn mutate(owner_ss58: &str, nft_id: u32, new_filename: &str, chain_api: Api<sr25519::Pair>) {
    let signer = get_pair_from_str(owner_ss58);
    let chain_api = chain_api.set_signer(sr25519_core::Pair::from(signer));
    // compose the extrinsic
    let offchain_uri = new_filename.as_bytes().to_vec();
    let xt = compose_extrinsic!(
        chain_api,
        "Nfts",
        "mutate",
        nft_id,
        offchain_uri,
        0u32,
        true
    );
    let tx_hash = chain_api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    info!("nft mutate extrinsic sent. Block Hash: {:?}", tx_hash);
    info!("waiting for confirmation of nft mutate");

    //subscribe to event Mutated
    let (events_in, events_out) = channel();
    chain_api.subscribe_events(events_in).unwrap();

    //Code to catch the mutated event and the errors coming from chain -> break infinite loop.
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
                                NFTEvent::Mutated(id) => {
                                    info!("Mutated event received");
                                    debug!("NFTId: {:?}", id);
                                    if *id == nft_id {
                                        break 'outer;
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
}
