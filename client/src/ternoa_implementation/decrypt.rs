use crate::ternoa_implementation::cipher::Key;
use crate::ternoa_implementation::LinesStorageHandler::LinesStorageHandler;
use sharks::{Share, Sharks};
use std::path::Path;

const SHAMIR_SHARES_DEFAULT_PATH: &str = "my_shamir_share";
const SHAMIR_SHARES_DEFAULT_LIST_FILENAME: &str = "shares.txt";

fn aes256key_from_shamir_shares(
    shamir_share_filename: &str,
    recovery_number_n: u8,
) -> Result<Key, String> {
    //read shamir shares from file -> return Vec(String) ?
    let shares_handler =
        LinesStorageHandler::new(SHAMIR_SHARES_DEFAULT_PATH, shamir_share_filename);
    let shares = shares_handler
        .read_shares_from_file()
        .map_err(|e| format!("Could not read shares: {}", e))?;
    let m_shares = shares.len() as usize;
    if m_shares < (recovery_number_n as usize) {
        return Err(format!("The threshold of shamir shards necessary for secret recovery (N = {:?}) must be smaller than the number of found shares (M = {:?})", recovery_number_n, m_shares));
    }

    let sharks = Sharks(recovery_number_n);
    let mut secret = sharks.recover(shares.as_slice()).unwrap();
    if secret.len() != 48 {
        return Err(format!("The recovered secret size doesn't correspond to the size of a Aes256 key (Found {:?} != (Aes256 {:?})", secret.len(), 48));
    }
    let iv = secret.drain(32..).collect();
    Ok((secret, iv))
}
