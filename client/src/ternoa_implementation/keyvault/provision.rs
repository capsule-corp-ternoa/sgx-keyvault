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
use crate::ternoa_implementation::cipher;
use my_node_primitives::NFTId;
use sharks::{Share, Sharks};

pub fn provision(
    keyvault_selection_file: &str,
    recovery_threshold: u8,
    _nft_id: NFTId,
    key_file: &str,
) -> Result<(), String> {
    // retrieve encryption key that is to be shamir shared to the keyvaults
    let encryption_key = get_key_from_file(key_file)?;
    // read urllist from file
    let url_handler = UrlStorageHandler::new().set_filename(keyvault_selection_file);
    let urls = url_handler
        .read_urls_from_file()
        .map_err(|e| format!("Could not read urls: {}", e))?;

    // create shamir shares
    let shamir_shares =
        create_shamir_shares(urls.len() as usize, recovery_threshold, &encryption_key)?;

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

/// Reads a key from a given file and concacenates the key to a single vector
fn get_key_from_file(key_file: &str) -> Result<Vec<u8>, String> {
    let key = cipher::recover_encryption_key(&cipher::keyfile_path(key_file))
        .map_err(|e| format!("Could not read key from file: {}", e))?;
    let mut concatenated = Vec::with_capacity(48);
    concatenated.extend_from_slice(&key.0);
    concatenated.extend_from_slice(&key.1);
    Ok(concatenated)
}

/// shamir split aes256 key into M shares, of which any N are needed for key recovery
fn create_shamir_shares(
    m_shares: usize,
    recovery_threshold: u8,
    secret: &[u8],
) -> Result<Vec<Share>, String> {
    // ensure m >= n
    if m_shares < (recovery_threshold as usize) {
        return Err(
            format!(
                "The threshold of shamir shards necessary for secret recovery (N = {:?}) must be smaller than the number of keyvaults (M = {:?})",
                recovery_threshold, m_shares
            )
        );
    }
    // Set a minimum threshold of n shares
    let sharks = Sharks(recovery_threshold);
    // Obtain an iterator over the shares for secret
    let dealer = sharks.dealer(secret);
    // create shares
    Ok(dealer.take(m_shares).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Tests for create shamir shares
    #[test]
    fn create_shamir_shares_works() {
        // given
        let m_shares: usize = 8;
        let recovery_number_n: u8 = 5;
        let secret: &[u8] = &[0, 1, 4, 6, 9, 0, 1, 20, 1];

        // when
        let shares = create_shamir_shares(m_shares, recovery_number_n, secret).unwrap();

        // then
        let sharks = Sharks(recovery_number_n);
        assert_eq!(secret, sharks.recover(shares.as_slice()).unwrap());
    }

    #[test]
    fn no_secret_recovery_with_too_few_shares() {
        // given
        let m_shares: usize = 20;
        let recovery_number_n: u8 = 19;
        let secret: &[u8] = &[0, 1, 4, 6, 9, 0, 1, 20, 1];
        let mut too_few_shares = Vec::new();

        // when
        let shares = create_shamir_shares(m_shares, recovery_number_n, secret).unwrap();

        for i in 0..(recovery_number_n - 3) {
            too_few_shares.push(shares[i as usize].clone());
        }

        // then
        let sharks = Sharks(recovery_number_n);
        assert!(sharks.recover(&too_few_shares).is_err());
    }

    #[test]
    fn shark_number_input_does_not_matter_when_recovering() {
        // given
        let m_shares: usize = 10;
        let recovery_number_n: u8 = 7;
        let secret: &[u8] = &[0, 1, 4, 6, 9, 0, 1, 20, 1];

        // when
        let shares = create_shamir_shares(m_shares, recovery_number_n, secret).unwrap();

        // then
        let sharks = Sharks(0);
        assert_eq!(secret, sharks.recover(shares.as_slice()).unwrap());
    }

    #[test]
    fn no_secret_recovery_when_m_smaller_n() {
        // given
        let m_shares: usize = 4;
        let recovery_number_n: u8 = 7;
        let secret: &[u8] = &[0, 1, 4, 6, 9, 0, 1, 20, 1];

        // when
        let shares = create_shamir_shares(m_shares, recovery_number_n, secret);

        // then
        assert!(shares.is_err());
    }

    #[test]
    fn get_key_from_file_concats_correctly() {
        // given
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("keyfile.aes256".to_owned());
        // generate key
        let key = cipher::recover_or_generate_encryption_key(&key_path).unwrap();

        // when
        let mut concat_key = get_key_from_file(key_path.to_str().unwrap()).unwrap();

        // then
        let iv: Vec<u8> = concat_key.drain(32..).collect();

        assert_eq!(key.0, concat_key);
        assert_eq!(key.1, iv);

        dir.close().unwrap();
    }
}
