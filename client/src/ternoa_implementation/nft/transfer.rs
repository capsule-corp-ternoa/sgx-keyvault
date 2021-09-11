use crate::{get_accountid_from_str, get_pair_from_str};
use codec::Decode;
use frame_system::Event as SystemEvent;
use log::*;
use my_node_primitives::nfts::NFTId;
use my_node_runtime::Event;
use sp_application_crypto::sr25519;
use sp_core::H256 as Hash;
use sp_core::{sr25519 as sr25519_core, Pair};
use sp_runtime::DispatchError;
use std::sync::mpsc::channel;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{
    compose_extrinsic, utils::FromHexString, Api, GenericAddress, XtStatus,
};
use ternoa_pallet_nfts::Event as NFTEvent;

///Transfer an NFT from an account to another one.
///Must be called by the current owner of the NFT.
pub fn transfer(from: &str, to: &str, nft_id: NFTId, chain_api: Api<sr25519::Pair, WsRpcClient>) {
    let signer = get_pair_from_str(from);
    let account_id = get_accountid_from_str(to);
    let chain_api = chain_api.set_signer(sr25519_core::Pair::from(signer));
    let to_id = GenericAddress::Id(account_id);
    info!("transfer the nft {} from {} to {}", nft_id, from, to);

    // compose the extrinsic
    let xt = compose_extrinsic!(chain_api, "Nfts", "transfer", nft_id, to_id);

    let tx_hash = chain_api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    info!("nft transfer extrinsic sent. Block Hash: {:?}", tx_hash);
    info!("waiting for confirmation of nft transfer");

    //subscribe to event Transfer
    let (events_in, events_out) = channel();
    chain_api.subscribe_events(events_in).unwrap();

    debug!("AccountId of signer  {:?}", get_accountid_from_str(from));

    //Code to catch the transfer event and the errors coming from chain -> break infinite loop.
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
                        Event::Nfts(nfte) => {
                            info!("NFT event received: {:?}", nfte);
                            match &nfte {
                                NFTEvent::Transfer(id, old_owner, new_owner) => {
                                    info!("Transfer event received");
                                    debug!("NFTId: {:?}", id);
                                    debug!("old owner accountId: {:?}", old_owner);
                                    debug!("new owner accountId: {:?}", new_owner);
                                    if nft_id == *id {
                                        break 'outer;
                                    }
                                }
                                _ => {
                                    debug!("ignoring unsupported NFT event");
                                }
                            }
                        }
                        Event::System(fse) => {
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
