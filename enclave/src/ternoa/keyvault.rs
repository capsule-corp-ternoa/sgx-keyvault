use crate::io;
use derive_more::{Display, From};
use log::*;
use my_node_primitives::AccountId;
use my_node_primitives::NFTId;
use std::fs;
use std::path::PathBuf;
use std::prelude::v1::*;
use std::sync::SgxRwLock;
use std::vec::Vec;
use substratee_stf::ShamirShare;

use super::nft_registry::NFTRegistryAuthorization;

#[derive(Debug, Display, From)]
pub enum Error {
    /// Wrapping of io error to keyvault error
    IoError(std::io::Error),
    ///Wrapping of Keyvault error
    KeyvaultError(String),
    /// Read lock error
    NFTRegistryLockPoisoned,
}

pub type Result<T> = std::result::Result<T, Error>;

pub const STORAGE_PATH: &str = "keyshare";

pub struct KeyvaultStorage<T: NFTRegistryAuthorization + 'static> {
    path: PathBuf,
    nft_access: &'static SgxRwLock<T>,
}

impl<T: NFTRegistryAuthorization> KeyvaultStorage<T> {
    pub fn new(t: &'static SgxRwLock<T>) -> Self {
        KeyvaultStorage {
            path: PathBuf::from(STORAGE_PATH),
            nft_access: t,
        }
    }
    pub fn set_path(mut self, path: PathBuf) -> Self {
        self.path = path;
        self
    }

    fn is_authorized(&self, owner: AccountId, nft_id: NFTId) -> Result<bool> {
        Ok(self
            .nft_access
            .read()
            .map_err(|_| Error::NFTRegistryLockPoisoned)?
            .is_authorized(owner, nft_id))
    }

    ///Store share on disk in sealed file
    pub fn provision(&self, owner: AccountId, nft_id: NFTId, share: ShamirShare) -> Result<()> {
        if !self.is_authorized(owner.clone(), nft_id)? {
            return Err(Error::KeyvaultError(format!(
                "Provision of {} is non authorized for this owner: {:?}.",
                nft_id, owner
            )));
        }
        self.seal(nft_id, share)?;
        Ok(())
    }

    ///Check if share for NFTId is in store
    pub fn check(&self, owner: AccountId, nft_id: NFTId) -> Result<bool> {
        if !self.is_authorized(owner.clone(), nft_id)? {
            error!(
                "check of {} is non authorized for this owner: {:?}.",
                nft_id, owner
            );
            return Ok(false);
        }
        let file_name = self.nft_sealed_file_path(nft_id);
        Ok(file_name.is_file())
    }

    ///Get the share from store for this NFTId
    pub fn get(&self, owner: AccountId, nft_id: NFTId) -> Result<Option<ShamirShare>> {
        if !&self.is_authorized(owner.clone(), nft_id)? {
            error!(
                "get of {} is non authorized for this owner: {:?}.",
                nft_id, owner
            );
            return Ok(None);
        }
        match self.unseal(nft_id) {
            Ok(share) => Ok(Some(share)),
            Err(e) => {
                error!("No share found for {:?} and NFT {}: {:?}", owner, nft_id, e);
                Ok(None)
            }
        }
    }

    ///Filename should encode the NFTId
    fn nft_sealed_file_path(&self, nft_id: NFTId) -> PathBuf {
        let name = format!("{}_Nft", nft_id);
        let mut p = PathBuf::from(&self.path);
        p.push(name);
        p.set_extension("bin");
        p
    }

    /// checks if the dir exists, and if not, creates a new one
    fn ensure_dir_exists(&self) -> Result<()> {
        if !&self.path.is_dir() {
            fs::create_dir_all(&self.path)?
        }
        Ok(())
    }

    ///Seal the share for NFTId
    fn seal(&self, nft_id: NFTId, share: ShamirShare) -> Result<()> {
        let filepath = self.nft_sealed_file_path(nft_id);
        if filepath.is_file() {
            warn!(
                "You will override an already existing sealed for {}!",
                nft_id
            );
        } else {
            //Directory will not be created by the seal method, so create it if it doesn't exist
            self.ensure_dir_exists()?;
        }

        match io::seal(share.as_slice(), &filepath.to_string_lossy()) {
            Ok(_r) => Ok(()),
            Err(e) => {
                return Err(Error::KeyvaultError(format!(
                    "Cannot seal {} : {:?}",
                    nft_id, e
                )));
            }
        }
    }

    ///Unseal the share for NFTId
    fn unseal(&self, nft_id: NFTId) -> Result<ShamirShare> {
        let filepath = self.nft_sealed_file_path(nft_id);
        match io::unseal(&filepath.to_string_lossy()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::KeyvaultError(format!(
                "Cannot unseal {} : {:?}",
                nft_id, e
            ))),
        }
    }
}

pub mod test {
    use super::*;
    use std::sync::atomic::{AtomicPtr, Ordering};
    use std::sync::Arc;

    //owner for test
    const ALICE_ENCODED: [u8; 32] = [
        212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88,
        133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125,
    ];

    static NFT_REGISTRY_TRUE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());
    static NFT_REGISTRY_FALSE: AtomicPtr<()> = AtomicPtr::new(0 as *mut ());

    struct MockNftAccess {
        return_value: bool,
    }

    impl MockNftAccess {
        fn author_true() -> &'static SgxRwLock<Self> {
            let ptr = NFT_REGISTRY_TRUE.load(Ordering::SeqCst) as *mut SgxRwLock<Self>;
            if ptr.is_null() {
                let storage_ptr = Arc::new(SgxRwLock::new(MockNftAccess { return_value: true }));
                NFT_REGISTRY_TRUE.store(Arc::into_raw(storage_ptr) as *mut (), Ordering::SeqCst);
                let ptr = NFT_REGISTRY_TRUE.load(Ordering::SeqCst) as *mut SgxRwLock<Self>;
                unsafe { &*ptr }
            } else {
                unsafe { &*ptr }
            }
        }

        fn author_false() -> &'static SgxRwLock<Self> {
            let ptr = NFT_REGISTRY_FALSE.load(Ordering::SeqCst) as *mut SgxRwLock<Self>;
            if ptr.is_null() {
                let storage_ptr = Arc::new(SgxRwLock::new(MockNftAccess {
                    return_value: false,
                }));
                NFT_REGISTRY_FALSE.store(Arc::into_raw(storage_ptr) as *mut (), Ordering::SeqCst);
                let ptr = NFT_REGISTRY_FALSE.load(Ordering::SeqCst) as *mut SgxRwLock<Self>;
                unsafe { &*ptr }
            } else {
                unsafe { &*ptr }
            }
        }
    }

    impl NFTRegistryAuthorization for MockNftAccess {
        fn is_authorized(&self, _owner: AccountId, _nft_id: NFTId) -> bool {
            self.return_value
        }
    }

    ///Tests
    pub fn create_keyvault_storage_works() {
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        assert_eq!(storage.path, PathBuf::from(STORAGE_PATH));
    }

    pub fn set_filename_and_path_works() {
        let dir = PathBuf::from("test_set_path");

        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());

        assert_eq!(storage.path, dir);
    }

    pub fn test_ensure_dir_exists_creates_new_if_not_existing() {
        let dir = PathBuf::from("test_creates_dir");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());
        storage.ensure_dir_exists().unwrap();

        assert!(dir.is_dir());

        //clean up
        fs::remove_dir_all(dir).unwrap();
    }

    pub fn test_sealed_file_name_contains_nftid() {
        let dir = PathBuf::from("test_sealed_file_name");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir);
        let file = storage.nft_sealed_file_path(197);
        let name = file.file_name().unwrap().to_str().unwrap();

        assert!(name.contains("197"));
    }

    pub fn test_seal_create_file() {
        let dir = PathBuf::from("test_seal_create_file");
        let share = Vec::from("hello");
        let nft_id = 365;
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());
        storage.seal(nft_id, share).unwrap();
        let file = PathBuf::from("test_seal_create_file/365_Nft.bin");

        assert!(file.is_file());

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    pub fn test_share_saved_in_sealed_file() {
        let dir = PathBuf::from("test_share_saved");
        let share = Vec::from("hello");
        let nft_id = 365;
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());
        storage.seal(nft_id, share.clone()).unwrap();

        let read_share = storage.unseal(nft_id).unwrap();
        for i in 1..5 {
            assert_eq!(share[i], read_share[i]);
        }

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    ///Can we override a seal file?
    pub fn test_seal_override_existing_sealed_file() {
        let dir = PathBuf::from("test_override_file");
        let share = Vec::from("hello");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());

        let nft_id = 5870;
        storage.seal(nft_id, share).unwrap();

        let new_share = Vec::from("hello_world");
        storage.seal(nft_id, new_share.clone()).unwrap();

        let read_share = storage.unseal(nft_id).unwrap();
        for i in 1..11 {
            assert_eq!(new_share[i], read_share[i]);
        }

        //clean up
        fs::remove_dir_all(dir.as_path()).unwrap();
    }

    pub fn test_unseal_fails_when_no_file_exists() {
        let dir = PathBuf::from("unseal_fails_no_file");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true()).set_path(dir.clone());
        let file = dir.join("365_Nft.bin");

        assert!(!file.is_file());

        assert!(storage.unseal(365).is_err());
    }

    pub fn test_provision_fails_when_no_nft_owner() {
        let nft_id = 5880;
        let author = AccountId::from(ALICE_ENCODED);
        let share = Vec::from("new_test_share_5875");
        let storage = KeyvaultStorage::new(MockNftAccess::author_false());
        assert!(storage.provision(author, nft_id, share).is_err());
    }

    pub fn test_check_is_false_when_no_nft_owner() {
        let nft_id = 5880;
        let author = AccountId::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new(MockNftAccess::author_false());
        assert!(!storage.check(author, nft_id).unwrap());
    }

    pub fn test_get_none_when_no_nft_owner() {
        let nft_id = 5880;
        let author = AccountId::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new(MockNftAccess::author_false());
        assert!(storage.get(author, nft_id).unwrap().is_none());
    }

    pub fn test_provision_store_share_in_sealed_file() {
        let nft_id = 5880;
        let author = AccountId::from(ALICE_ENCODED);
        let share = Vec::from("new_test_share_5875");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        storage.provision(author, nft_id, share.clone()).unwrap();

        let file_name = storage.nft_sealed_file_path(nft_id);
        assert!(file_name.is_file());

        let read_share = storage.unseal(nft_id).unwrap();
        for i in 1..18 {
            assert_eq!(share[i], read_share[i]);
        }

        //Clean-up
        fs::remove_file(file_name).unwrap();
    }

    pub fn test_check_is_true_when_sealed_file() {
        let nft_id = 5890;
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        let file_name = storage.nft_sealed_file_path(nft_id);
        let author = AccountId::from(ALICE_ENCODED);
        let new_share = Vec::from("hello_world");

        storage.seal(nft_id, new_share).unwrap();

        assert!(storage.check(author, nft_id).unwrap());

        //Clean-up
        fs::remove_file(file_name).unwrap();
    }

    pub fn test_check_is_false_when_no_sealed_file() {
        let nft_id = 6000;
        let author = AccountId::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        assert!(!storage.check(author, nft_id).unwrap());
    }

    pub fn test_get_none_when_nft_not_in_store() {
        let nft_id = 6010;
        let author = AccountId::from(ALICE_ENCODED);
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        assert! {storage.get(author, nft_id).unwrap().is_none()};
    }

    pub fn test_get_the_valid_stored_share() {
        let nft_id = 6020;
        let owner = AccountId::from(ALICE_ENCODED);
        let share = Vec::from("new_test_share_6020");
        let storage = KeyvaultStorage::new(MockNftAccess::author_true());
        storage
            .provision(owner.clone(), nft_id, share.clone())
            .unwrap();

        let read_share = storage.get(owner, nft_id).unwrap().unwrap();
        for i in 1..18 {
            assert_eq!(share[i], read_share[i]);
        }

        //Clean-up
        let file_name = storage.nft_sealed_file_path(nft_id);
        fs::remove_file(file_name).unwrap();
    }
}
