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

//!A Trait for AES Encryption
//! An implementation for AES256

use aes::Aes256;
use derive_more::{Display, From};
use log::*;
use ofb::stream_cipher::{InvalidKeyNonceLength, NewStreamCipher, StreamCipher, SyncStreamCipher};
use ofb::Ofb;
use rand::{thread_rng, Rng};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};

pub const DEFAULT_OUT_NAME: &str = "output_file";

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

pub trait AesEncryption {
    type Key;
    type Error: From<Error>;
    type Cipher: StreamCipher;

    fn create_sealed_if_absent(&self) -> Result<()>;
    fn create_sealed(&self) -> Result<()>;
    fn read_sealed(&self) -> Result<Self::Key>;
    fn de_or_encrypt(&self, bytes: &mut Vec<u8>) -> Result<()>;
    fn de_or_encrypt_file(&self, in_filename: &String) -> Result<()>;
}

///Encrypt a file with AES256 and save it in the same folder as file_name.ciphertext
///Create a key and save it into a file it in the same folder as file_name.aes256
pub fn encrypt_aes256(filename: String) -> Result<()> {
    //Key and output files:
    let mut split = filename.split('.');
    let name = split.next(); //filename without extension

    let entcrypted_file = name.unwrap_or(DEFAULT_OUT_NAME).to_owned() + ".ciphertext";
    let aes = name.unwrap_or(DEFAULT_OUT_NAME).to_owned() + ".aes256";

    let output_file = entcrypted_file.clone();
    let key_file = aes.clone();

    let encrypt = Aes256Encryption {
        key_file,
        output_file,
    };

    //create key and save file
    AesEncryption::create_sealed_if_absent(&encrypt)?;
    //Encrypt file
    AesEncryption::de_or_encrypt_file(&encrypt, &filename)?;

    //Test
    //decrypt_aes256(entcrypted_file.clone())?;
    Ok(())
}

pub fn decrypt_aes256(filename: String) -> Result<()> {
    //Key and output files:
    let mut split = filename.split('.');
    let name = split.next(); //filename without extension

    let decrypted_file = name.unwrap_or(DEFAULT_OUT_NAME).to_owned() + ".decrypted";
    let aes = name.unwrap_or(DEFAULT_OUT_NAME).to_owned() + ".aes256";

    let output_file = decrypted_file.clone();
    let key_file = aes.clone();

    let decrypt = Aes256Encryption {
        key_file,
        output_file,
    };
    AesEncryption::de_or_encrypt_file(&decrypt, &filename)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct Aes256Encryption {
    pub key_file: String,
    pub output_file: String,
}

impl AesEncryption for Aes256Encryption {
    type Key = (Vec<u8>, Vec<u8>);
    type Error = Error;
    type Cipher = Ofb<Aes256>;

    ///Create a key and a iv, if there is none
    fn create_sealed_if_absent(&self) -> Result<()> {
        if let Err(_e) = File::open(&self.key_file) {
            debug!("Keyfile not found, creating new! {}", &self.key_file);
            Self::create_sealed(&self)?;
        }
        Ok(())
    }

    /// Create the key and iv (random) //For AES256 : key :32, iv 16
    fn create_sealed(&self) -> Result<()> {
        let mut key = [0u8; 32];
        thread_rng().try_fill(&mut key)?;
        let mut iv = [0u8; 16];
        thread_rng().try_fill(&mut iv)?;
        let mut key_iv = key.to_vec();
        key_iv.extend_from_slice(&iv);
        fs::File::create(&self.key_file).map(|mut f| f.write(&key_iv))?;
        Ok(())
    }

    ///Read the key and the iv from the file
    fn read_sealed(&self) -> Result<Self::Key> {
        let mut file = File::open(&self.key_file)?;
        let mut buffer = [0u8; 48];
        file.read(&mut buffer)?;
        let key: Vec<u8> = buffer[..32].to_vec();
        let inv = buffer[32..].to_vec();
        Ok((key, inv))
    }

    /// If AES acts on the encrypted data it decrypts and vice versa
    /// Key and iv are in the block
    fn de_or_encrypt(&self, bytes: &mut Vec<u8>) -> Result<()> {
        Self::read_sealed(&self)
            .map(|(key, iv)| Self::Cipher::new_var(&key, &iv))?
            .map(|mut ofb| ofb.apply_keystream(bytes))?;
        Ok(())
    }

    /// If AES acts on the encrypted file it decrypts and vice versa
    /// Key and iv are not in the file
    fn de_or_encrypt_file(&self, in_filename: &String) -> Result<()> {
        //Get the symmetric key from file
        let aes = Self::read_sealed(&self)?;

        // create cipher instance
        let mut cipher = Self::Cipher::new_var(&*aes.0, &*aes.1)?;

        //Read file
        let mut f = File::open(in_filename)?;
        let mut buffer = Vec::new();

        // read the whole file
        f.read_to_end(&mut buffer)?;

        //stream ciphers
        for chunk in buffer.chunks_mut(16) {
            cipher.apply_keystream(chunk);
        }

        //Save file
        fs::File::create(&self.output_file).map(|mut f| f.write(&buffer))?;

        Ok(())
    }
}
