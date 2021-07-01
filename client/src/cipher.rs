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

///A module to encrypt or decrypt file with AES256
use aes::Aes256;
use derive_more::{Display, From};
use log::*;
use ofb::stream_cipher::{InvalidKeyNonceLength, NewStreamCipher, SyncStreamCipher};
use ofb::Ofb;
use rand::{thread_rng, Rng};
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
}

pub type Result<T> = std::result::Result<T, Error>;

///Symmetric keyKey & iv
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


///Read the encryption key from the file and generate one if there is none
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
        },
        Ok(key) => Ok(key)
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
        None => {
            recover_or_generate_encryption_key(
                &keyfile_path(plaintext_filename)
            )?
        }
        Some(key) => key,
    };

    de_or_encrypt_file(
        &Path::new(plaintext_filename), 
        &ciphertext_path(plaintext_filename), 
        encryption_key
    )
}

///Decrypt a file with AES256 and save it in the same folder as file_name.decrypted
///Create a key and save it into a file it in the same folder as file_name.aes256 if key is None
pub fn decrypt(ciphertext_filename: &str, key: Option<Key>) -> Result<()> {

    let encryption_key = match key {
        None => {
            recover_encryption_key(
                &keyfile_path(ciphertext_filename)
            )?
        }
        Some(key) => key,
    };

    de_or_encrypt_file(
        &Path::new(ciphertext_filename), 
        &decrypted_path(ciphertext_filename), 
        encryption_key
    )
}
