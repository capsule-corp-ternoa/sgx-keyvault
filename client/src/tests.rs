use crate::cipher::{decrypt, encrypt, recover_or_generate_encryption_key};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

const TEST_KEY_FILE_EXT: &str = "aes256";
const TEST_EN_FILE_EXT: &str = "ciphertext";
const TEST_DE_FILE_EXT: &str = "decrypted";
const TEST_TEXT: &str = "I'm nobody! Who are you?\nAre you nobody, too?";

fn create_test_file(dir_path: &Path) -> std::io::Result<PathBuf> {
    let file_path = dir_path.join("input.txt");
    let mut test_file = File::create(file_path.clone()).unwrap();
    write!(test_file, "{}", TEST_TEXT).unwrap();
    Ok(file_path)
}

fn try_read_file(decrypted_file_path: &str) -> std::io::Result<String> {
    let decrypted_file_result = File::open(decrypted_file_path);
    assert!(decrypted_file_result.is_ok());
    let decrypted_file = decrypted_file_result.ok().unwrap();
    fs::read_to_string(decrypted_file_path)
}

fn assert_decrypted_file_is_valid(decrypted_file_path: &str) {
    let decrypted_read_result = try_read_file(decrypted_file_path);
    assert!(decrypted_read_result.is_ok());
    let text = decrypted_read_result.ok().unwrap();
    assert_eq!(text, TEST_TEXT);
}

#[test]
fn verify_recover_or_generate_encryption_key() {
    let dir = tempdir().unwrap();
    let key_file_path = dir.path().join("keyfile.".to_owned() + TEST_KEY_FILE_EXT);
    assert!(!key_file_path.exists());

    let result = recover_or_generate_encryption_key(&key_file_path);
    assert!(result.is_ok());
    assert!(key_file_path.exists());

    let aes = result.ok().unwrap();
    assert_eq!(aes.0.len(), 32);
    assert_eq!(aes.1.len(), 16);

    let re_result = recover_or_generate_encryption_key(&key_file_path);
    assert!(re_result.is_ok());
    assert!(key_file_path.exists());

    dir.close();
}

#[test]
fn verify_encrypt_default_key() {
    let dir = tempdir().unwrap();
    let en_file_path = dir.path().join("input.".to_owned() + TEST_EN_FILE_EXT);
    let de_file_path = dir.path().join("input.".to_owned() + TEST_DE_FILE_EXT);
    let key_file_path = dir.path().join("input.".to_owned() + TEST_KEY_FILE_EXT);

    //Create test input file
    let test_file_path = create_test_file(dir.path()).ok().unwrap();
    assert!(test_file_path.exists());

    let encrypt_result = encrypt(test_file_path.to_str().unwrap(), None);
    assert!(encrypt_result.is_ok());
    assert!(en_file_path.exists());
    assert!(key_file_path.exists());

    let decrypt_result = decrypt(en_file_path.to_str().unwrap(), None);
    assert!(decrypt_result.is_ok());
    assert!(de_file_path.exists());
    assert!(key_file_path.exists());

    assert_decrypted_file_is_valid(de_file_path.to_str().unwrap());

    dir.close();
}

#[test]
fn verify_encrypt_with_key() {
    let dir = tempdir().unwrap();
    let en_file_path = dir.path().join("input.".to_owned() + TEST_EN_FILE_EXT);
    let de_file_path = dir.path().join("input.".to_owned() + TEST_DE_FILE_EXT);
    let key_file_path = dir.path().join("keyfile.".to_owned() + TEST_KEY_FILE_EXT);

    //Create test input file
    let test_file_path = create_test_file(dir.path()).ok().unwrap();
    assert!(test_file_path.exists());

    //generate key
    let result = recover_or_generate_encryption_key(&key_file_path);
    assert!(result.is_ok());
    assert!(key_file_path.exists());
    let aes = result.ok().unwrap();

    //encrypt
    let encrypt_result = encrypt(test_file_path.to_str().unwrap(), Some(aes.clone()));
    assert!(encrypt_result.is_ok());
    assert!(en_file_path.exists());

    let decrypt_result = decrypt(en_file_path.to_str().unwrap(), Some(aes));
    assert!(decrypt_result.is_ok());
    assert!(de_file_path.exists());

    //Decrypted file
    assert_decrypted_file_is_valid(de_file_path.to_str().unwrap());

    dir.close();
}

#[test]
fn verify_decrypt_fails_without_keyfile() {
    let dir = tempdir().unwrap();

    let file_path = dir.path().join("tmp");
    let mut test_file = File::create(file_path.clone()).unwrap();
    writeln!(test_file, "blablabla").unwrap();

    assert!(encrypt(file_path.to_str().unwrap(), None).is_ok());

    let key_path = file_path.join("tmp.").join(TEST_KEY_FILE_EXT);
    fs::remove_file(key_path);

    let ciphertext_path = file_path.join("tmp.").join(TEST_EN_FILE_EXT);

    assert!(decrypt(ciphertext_path.to_str().unwrap(), None).is_err());
    dir.close();
}

#[test]
fn verify_error_encrypt_decrypt_diff_keys() {
    let dir = tempdir().unwrap();
    let en_file_path = dir.path().join("input.".to_owned() + TEST_EN_FILE_EXT);
    let de_file_path = dir.path().join("input.".to_owned() + TEST_DE_FILE_EXT);
    let key_file_path = dir.path().join("keyfile.".to_owned() + TEST_KEY_FILE_EXT);
    let key_file_path_2 = dir.path().join("keyfile2.".to_owned() + TEST_KEY_FILE_EXT);

    //Create test input file
    let test_file_path = create_test_file(dir.path()).ok().unwrap();
    assert!(test_file_path.exists());

    //generate 1. key
    let result = recover_or_generate_encryption_key(&key_file_path);
    assert!(result.is_ok());
    assert!(key_file_path.exists());
    let aes = result.ok().unwrap();

    //encrypt
    let encrypt_result = encrypt(test_file_path.to_str().unwrap(), Some(aes));
    assert!(encrypt_result.is_ok());
    assert!(en_file_path.exists());

    //generate 2. key
    let result_key2 = recover_or_generate_encryption_key(&key_file_path_2);
    assert!(result_key2.is_ok());
    assert!(key_file_path_2.exists());
    let aes_2 = result_key2.ok().unwrap();

    let decrypt_result = decrypt(en_file_path.to_str().unwrap(), Some(aes_2));
    assert!(decrypt_result.is_ok());
    assert!(de_file_path.exists());

    //Decrypted file isn't valid
    let decrypted_read_result = try_read_file(de_file_path.to_str().unwrap());

    assert!(decrypted_read_result.is_err());

    dir.close();
}
