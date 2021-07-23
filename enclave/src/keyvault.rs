use crate::io;
use crate::utils::UnwrapOrSgxErrorUnexpected;
use my_node_primitives::NFTId;
use sgx_types::{sgx_status_t, SgxResult};
use sp_core::crypto::AccountId32;
use std::fs;
use std::path::PathBuf;
use std::prelude::v1::*;
use std::vec::Vec;

pub const STORAGE_PATH: &str = "keyshare";

//owner for test
const ALICE_ENCODED: [u8; 32] = [
    212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133,
    76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
];

pub struct KeyvaultStorage {}

//TODO: check if import of crate shark:share is possible. Wait for haerdib answer
impl KeyvaultStorage {
    ///Store share on disk in sealed file
    pub fn provision(owner: AccountId32, nft_id: NFTId, share: Vec<u8>) -> SgxResult<()> {
        if !is_authorized(owner, nft_id) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_SIGNATURE);
        }
        seal(STORAGE_PATH, nft_id, share)?;
        Ok(())
    }

    ///Check if share for NFTId is in store
    pub fn check(owner: AccountId32, nft_id: NFTId) -> bool {
        //TODO Authorization owner
        if !is_authorized(owner, nft_id) {
            return false;
        }
        let file_name = sealed_filepath(STORAGE_PATH, nft_id);
        file_name.unwrap().is_file()
    }

    ///Get the share from store for this NFTId
    pub fn get(owner: AccountId32, nft_id: NFTId) -> SgxResult<Vec<u8>> {
        //TODO Authorization owner
        if !is_authorized(owner, nft_id) {
            return Err(sgx_status_t::SGX_ERROR_INVALID_SIGNATURE);
        }
        unseal(STORAGE_PATH, nft_id)
    }
}
fn is_authorized(owner: AccountId32, nft_id: NFTId) -> bool {
    //TODO Authorization owner
    true
}

///Filename should encode the NFTId
fn sealed_filepath(path: &str, nft_id: NFTId) -> SgxResult<PathBuf> {
    let name = format!("{}_Nft", nft_id);
    let mut p = PathBuf::from(path);
    p.push(name);
    p.set_extension("bin");
    Ok(p)
}

/// checks if the dir exists, and if not, creates a new one
fn ensure_dir_exists(dir: PathBuf) -> SgxResult<sgx_status_t> {
    if !dir.is_dir() {
        fs::create_dir_all(&dir).sgx_error_with_log(&format!(
            "[Enclave] Keyvault, creating dir '{}' failed",
            dir.to_str().unwrap()
        ))?
    }
    Ok(sgx_status_t::SGX_SUCCESS)
}

///Seal the share for NFTId
fn seal(dir: &str, nft_id: NFTId, share: Vec<u8>) -> SgxResult<sgx_status_t> {
    let filepath = sealed_filepath(dir, nft_id)?;
    //Directory will not be created by the seal method, so create it if it doesn't exist
    let p = PathBuf::from(dir);
    ensure_dir_exists(p)?;
    io::seal(share.as_slice(), filepath.to_str().unwrap())
}

///Unseal the share for NFTId
fn unseal(dir: &str, nft_id: NFTId) -> SgxResult<Vec<u8>> {
    let filepath = sealed_filepath(dir, nft_id)?;
    let share = io::unseal(filepath.to_str().unwrap())?;
    Ok(share)
}

///Tests
pub fn ensure_dir_exists_creates_new_if_not_existing() {
    let dir = PathBuf::from("test_creates_dir");

    ensure_dir_exists(dir.clone()).unwrap();

    assert!(dir.is_dir());

    //clean up
    fs::remove_dir_all(dir).unwrap();
}

pub fn test_sealed_file_encode_nftid() {
    let dir = "test_sealed_file_name";
    let file = sealed_filepath(dir, 197).unwrap();
    let name = file.file_name().unwrap().to_str().unwrap();

    assert!(name.contains("197"));
}

pub fn test_share_saved_in_sealed_file() {
    let dir = "test_share_saved";
    let share = Vec::from("hello");
    let nft_id = 365;
    seal(dir, nft_id, share.clone()).unwrap();

    let read_share = unseal(dir, nft_id).unwrap();

    for i in 1..5 {
        assert_eq!(share[i], read_share[i]);
    }

    //clean up
    fs::remove_dir_all(dir).unwrap();
}

///Can we override a seal file?
pub fn test_seal_override_existing_sealed_file() {
    let dir = "test_override_file";
    let share = Vec::from("hello");
    let nft_id = 5870;
    seal(dir, nft_id, share).unwrap();

    let new_share = Vec::from("hello_world");
    seal(dir, nft_id, new_share.clone()).unwrap();

    let read_share = unseal(dir, nft_id).unwrap();

    for i in 1..11 {
        assert_eq!(new_share[i], read_share[i]);
    }

    //clean up
    fs::remove_dir_all(dir).unwrap();
}

pub fn test_provision_fails_when_no_nft_owner() {
    //TODO
}

pub fn test_check_is_false_when_no_nft_owner() {
    //TODO
}

pub fn test_get_fails_when_no_nft_owner() {
    //TODO
}

pub fn test_provision_store_share_in_sealed_file() {
    let nft_id = 5880;
    let author = AccountId32::from(ALICE_ENCODED);
    let share = Vec::from("new_test_share_5875");

    KeyvaultStorage::provision(author, nft_id, share.clone()).unwrap();

    let file_name = sealed_filepath(STORAGE_PATH, nft_id).unwrap();
    assert!(file_name.is_file());

    let read_share = unseal(STORAGE_PATH, nft_id).unwrap();
    for i in 1..18 {
        assert_eq!(share[i], read_share[i]);
    }

    //Clean-up
    //  fs::remove_file(file_name).unwrap();
}

pub fn test_check_is_true_when_sealed_file() {
    let nft_id = 5890;
    let file_name = sealed_filepath(STORAGE_PATH, nft_id).unwrap();
    let author = AccountId32::from(ALICE_ENCODED);

    let new_share = Vec::from("hello_world");
    seal(STORAGE_PATH, nft_id, new_share.clone()).unwrap();

    assert!(KeyvaultStorage::check(author, nft_id));

    //Clean-up
    fs::remove_file(file_name).unwrap();
}

pub fn test_check_is_false_when_no_sealed_file() {
    let nft_id = 6000;
    let author = AccountId32::from(ALICE_ENCODED);

    assert!(!KeyvaultStorage::check(author, nft_id));
}

pub fn test_get_fails_when_nft_not_in_store() {
    let nft_id = 6010;
    let author = AccountId32::from(ALICE_ENCODED);

    assert! {KeyvaultStorage::get(author, nft_id).is_err()};
}

pub fn test_get_the_valid_stored_share() {
    let nft_id = 6020;
    let owner = AccountId32::from(ALICE_ENCODED);
    let share = Vec::from("new_test_share_6020");

    KeyvaultStorage::provision(owner.clone(), nft_id, share.clone()).unwrap();

    let read_share = KeyvaultStorage::get(owner, nft_id).unwrap();

    for i in 1..18 {
        assert_eq!(share[i], read_share[i]);
    }

    //Clean-up
    let file_name = sealed_filepath(STORAGE_PATH, nft_id).unwrap();
    fs::remove_file(file_name).unwrap();
}
