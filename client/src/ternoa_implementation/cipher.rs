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

use crate::ternoa_implementation::cipher::Error::ShamirError;
use crate::ternoa_implementation::local_storage_handler::{LocalFileStorage, VecToLinesConverter};
///A module to encrypt or decrypt file with AES256
use aes::Aes256;
use derive_more::{Display, From};
use log::*;
use ofb::stream_cipher::{InvalidKeyNonceLength, NewStreamCipher, SyncStreamCipher};
use ofb::Ofb;
use rand::{thread_rng, Rng};
use sharks::{Share, Sharks};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Display, From)]
pub enum Error {
    /// Wrapping of io error to Aes Error
    IoError(std::io::Error),
    /// Wrapping of InvalidKeyNonceLength to Aes Error
    KeyStream(InvalidKeyNonceLength),
    ///Wrapping of rand::Error to Aes Error
    RandError(rand::Error),
    ///Wrapping of Shamir's share error
    ShamirError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

///Symmetric key Key & iv
pub type Key = (Vec<u8>, Vec<u8>);

///Cipher for AES 256
type AesOfb = Ofb<Aes256>;

/// Create the key and iv (random) //For AES256 : key :32, iv 16
fn create_symmetric_key() -> Result<Key> {
    let mut key = [0u8; 32];
    thread_rng().try_fill(&mut key)?;
    let mut iv = [0u8; 16];
    thread_rng().try_fill(&mut iv)?;
    Ok((key.to_vec(), iv.to_vec()))
}

///Read the encryption key from a file or generate one if there is none
pub fn recover_or_generate_encryption_key(key_filename: &Path) -> Result<Key> {
    match recover_encryption_key(&key_filename) {
        Err(e) => {
            debug!(
                "Key recovery error {}, creating new! {}",
                e,
                key_filename.display()
            );
            let key_iv = create_symmetric_key()?;
            let mut buf = key_iv.clone().0;
            buf.extend_from_slice(&key_iv.1);
            let mut key_file = File::create(key_filename)?;
            key_file.write_all(&buf)?;
            Ok((key_iv.0, key_iv.1))
        }
        Ok(key) => Ok(key),
    }
}

///Read the encryption key from the file
pub fn recover_encryption_key(key_filename: &Path) -> Result<Key> {
    let mut file = File::open(&key_filename)?;
    let mut buffer = [0u8; 48];
    file.read_exact(&mut buffer)?;
    let key = buffer[..32].to_vec();
    let iv = buffer[32..].to_vec();
    Ok((key, iv))
}

///Get the shamir shares from a file
pub fn shamir_shares_from_file(shamir_share_filename: PathBuf) -> Result<Vec<Share>> {
    let dir = match shamir_share_filename.parent() {
        Some(path) => path,
        None => {
            return Err(ShamirError(format!(
                "The shamir share file is invalid. No parent directory : {}.",
                shamir_share_filename.as_path().to_str().unwrap()
            )))
        }
    };

    let filename = match shamir_share_filename.file_name() {
        Some(path) => path,
        None => {
            return Err(ShamirError(format!(
                "The shamir share file is invalid. No valid filename : {}.",
                shamir_share_filename.as_path().to_str().unwrap()
            )))
        }
    };

    //read shamir shares from file -> return Vec(String) ?
    let shares_handler = LocalFileStorage::new(PathBuf::from(dir), PathBuf::from(filename));

    shares_handler
        .read_lines()
        .map_err(|e| ShamirError(format!("Could not read shares: {}", e)))
}

///Recover the encryption key from shamir shares
pub fn aes256key_from_shamir_shares(shares: Vec<Share>) -> Result<Key> {
    if shares.is_empty() {
        return Err(ShamirError(
            "No shamir shares to recover the encryption key ".to_string(),
        ));
    }
    //No need of threshold shares num to recover key
    let sharks = Sharks(0);
    let mut secret = sharks.recover(shares.as_slice()).unwrap();
    if secret.len() != 48 {
        return Err(
            ShamirError(
                format!("The recovered secret size doesn't correspond to the size of a Aes256 key (Found {:?} != (Aes256 {:?})", secret.len(), 48)
            )
        );
    }
    let iv = secret.drain(32..).collect();
    debug!("Found secret : {:?},{:?}", secret, iv);
    Ok((secret, iv))
}

/// If AES acts on the encrypted file it decrypts and vice versa
/// Key and iv are not in the file
fn de_or_encrypt_file(input_file: &Path, output_file: &Path, key: Key) -> Result<()> {
    // create cipher instance
    let mut cipher = AesOfb::new_var(&*key.0, &*key.1)?;

    //Read file
    let mut f = File::open(input_file)?;
    let mut buffer = Vec::new();

    // read the whole file
    f.read_to_end(&mut buffer)?;

    //stream ciphers
    for chunk in buffer.chunks_mut(16) {
        cipher.apply_keystream(chunk);
    }

    //Save file
    let mut file = File::create(output_file)?;
    file.write_all(&buffer)?;

    Ok(())
}

fn ciphertext_path(plaintext_filename: &str) -> PathBuf {
    let mut path = PathBuf::from(plaintext_filename);
    path.set_extension("ciphertext");
    path
}

#[allow(dead_code)]
fn decrypted_path(ciphertext_filename: &str) -> PathBuf {
    let mut path = PathBuf::from(ciphertext_filename);
    path.set_extension("decrypted");
    path
}

fn keyfile_path(plaintext_filename: &str) -> PathBuf {
    let mut path = PathBuf::from(plaintext_filename);
    path.set_extension("aes256");
    path
}

///Encrypt a file with AES256 and save it in the same folder as file_name.ciphertext
///Create a key and save it into a file it in the same folder as file_name.aes256 if key is None
pub fn encrypt(plaintext_filename: &str, key: Option<Key>) -> Result<()> {
    let encryption_key = match key {
        None => recover_or_generate_encryption_key(&keyfile_path(plaintext_filename))?,
        Some(key) => key,
    };
    de_or_encrypt_file(
        &Path::new(plaintext_filename),
        &ciphertext_path(plaintext_filename),
        encryption_key,
    )
}

#[allow(dead_code)]
///Decrypt a file with AES256 and save it in the same folder as file_name.decrypted
///if key is None, recover it from the file ciphertext_filename.aes256
pub fn decrypt_with_key(ciphertext_filename: &str, key: Option<Key>) -> Result<()> {
    let encryption_key = match key {
        None => recover_encryption_key(&keyfile_path(ciphertext_filename))?,
        Some(key) => key,
    };

    de_or_encrypt_file(
        &Path::new(ciphertext_filename),
        &decrypted_path(ciphertext_filename),
        encryption_key,
    )
}

///Decrypt a file with a key recover from shamir shares
///Assume that the shamir_share_file provides the correct number of shares needed for recovery
pub fn decrypt(ciphertext_filename: &str, shamir_share_file: &str) -> Result<()> {
    let shares = shamir_shares_from_file(PathBuf::from(shamir_share_file))?;
    let encryption_key = aes256key_from_shamir_shares(shares)?;

    de_or_encrypt_file(
        &Path::new(ciphertext_filename),
        &decrypted_path(ciphertext_filename),
        encryption_key,
    )
}
