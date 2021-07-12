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
use super::url_storage_handler::UrlStorageHandler;
use sharks::{ Sharks, Share };
use log::*;

pub fn provision(path: &str, filename: &str, recovery_number_n: u8) -> Result<(), String> {
    // TODO: how / from where to read aes256 key -> wait for PR of issue #1?
    let secret = &[0u8,4];
    // read urllist from file
    let url_handler = UrlStorageHandler::open(path, filename)
        .map_err(|e| format!("Could not access directory: {}", e))?;
    let urls = url_handler.read_urls_from_file()
        .map_err(|e| format!("Could not read urls: {}", e))?;

    // create shamir shares
    let shamir_shares = create_shamir_shares(urls.len() as usize, recovery_number_n, secret);

    // for all urls in list (= # of shares):
    //    a. send ith share to url_i
    //    b. verify availability
    for _shamir_share in shamir_shares.iter() {
       // send to enclave:
        // TODO: TASK of ISSUE #6
    }

    // TODO: create file NFT urllist NFT File
    Ok(())

}

/// shamir split aes256 key into M shares, of which any N are needed for key recovery
fn create_shamir_shares(m_shares: usize, recovery_number_n: u8, secret: &[u8]) -> Vec<Share> {
    // Set a minimum threshold of 10 shares
    let sharks = Sharks(recovery_number_n);
    // Obtain an iterator over the shares for secret [1, 2, 3, 4]
    let dealer = sharks.dealer(secret);
    // create shares
    let shares: Vec<Share> = dealer.take(m_shares).collect();
    debug!("Recovered secret: {:?}", sharks.recover(shares.as_slice()).unwrap());
    shares
}
