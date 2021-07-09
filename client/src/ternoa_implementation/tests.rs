use crate::ternoa_implementation::cipher::{decrypt, encrypt, recover_or_generate_encryption_key};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

const KEYFILE_EXT: &str = "aes256";
const CIPHERTEXT_EXT: &str = "ciphertext";
const DECRYPTED_EXT: &str = "decrypted";
const TEST_TEXT: &str = "I'm nobody! Who are you?\nAre you nobody, too?";

fn plaintext_input(dir_path: &Path) -> std::io::Result<PathBuf> {
    let file_path = dir_path.join("input.txt");
    let mut test_file = File::create(file_path.clone()).unwrap();
    write!(test_file, "{}", TEST_TEXT).unwrap();
    Ok(file_path)
}

fn decrypted_text(decrypted_file_path: &str) -> String {
    let decrypted_read_result = fs::read_to_string(decrypted_file_path);
    let text = decrypted_read_result.ok().unwrap();
    text
}

#[test]
fn verify_recover_or_generate_encryption_key() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    assert!(!key_path.exists());

    let result = recover_or_generate_encryption_key(&key_path);
    assert!(result.is_ok());
    assert!(key_path.exists());

    let aes = result.ok().unwrap();
    assert_eq!(aes.0.len(), 32);
    assert_eq!(aes.1.len(), 16);

    let re_result = recover_or_generate_encryption_key(&key_path);
    assert!(re_result.is_ok());
    assert!(key_path.exists());

    dir.close();
}

#[test]
fn verify_encrypt_default_key() {
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("input.".to_owned() + KEYFILE_EXT);

    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();
    assert!(inputfile_path.exists());

    let encrypt_result = encrypt(inputfile_path.to_str().unwrap(), None);
    assert!(encrypt_result.is_ok());
    assert!(ciphertext_path.exists());
    assert!(key_path.exists());

    let decrypt_result = decrypt(ciphertext_path.to_str().unwrap(), None);
    assert!(decrypt_result.is_ok());
    assert!(decrypted_path.exists());
    assert!(key_path.exists());

    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_encrypt_with_key() {
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);

    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    assert!(test_file_path.exists());

    //generate key
    let result = recover_or_generate_encryption_key(&key_path);
    assert!(result.is_ok());
    assert!(key_path.exists());
    let aes = result.ok().unwrap();

    //encrypt
    let encrypt_result = encrypt(test_file_path.to_str().unwrap(), Some(aes.clone()));
    assert!(encrypt_result.is_ok());
    assert!(ciphertext_path.exists());

    let decrypt_result = decrypt(ciphertext_path.to_str().unwrap(), Some(aes));
    assert!(decrypt_result.is_ok());
    assert!(decrypted_path.exists());

    //Decrypted file
    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_decrypt_fails_without_keyfile() {
    let dir = tempdir().unwrap();

    let file_path = dir.path().join("tmp");
    let mut test_file = File::create(file_path.clone()).unwrap();
    writeln!(test_file, "blablabla").unwrap();

    assert!(encrypt(file_path.to_str().unwrap(), None).is_ok());

    let key_path = file_path.join("tmp.").join(KEYFILE_EXT);
    fs::remove_file(key_path);

    let ciphertext_path = file_path.join("tmp.").join(CIPHERTEXT_EXT);

    assert!(decrypt(ciphertext_path.to_str().unwrap(), None).is_err());
    dir.close();
}

#[test]
fn verify_error_encrypt_decrypt_diff_keys() {
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    let other_key_path = dir.path().join("keyfile2.".to_owned() + KEYFILE_EXT);

    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    assert!(test_file_path.exists());

    //generate 1. key
    let result = recover_or_generate_encryption_key(&key_path);
    assert!(result.is_ok());
    assert!(key_path.exists());
    let aes = result.ok().unwrap();

    //encrypt
    let encrypt_result = encrypt(test_file_path.to_str().unwrap(), Some(aes));
    assert!(encrypt_result.is_ok());
    assert!(ciphertext_path.exists());

    //generate 2. key
    let result_key2 = recover_or_generate_encryption_key(&other_key_path);
    assert!(result_key2.is_ok());
    assert!(other_key_path.exists());
    let other_aes = result_key2.ok().unwrap();

    let decrypt_result = decrypt(ciphertext_path.to_str().unwrap(), Some(other_aes));
    assert!(decrypt_result.is_ok());
    assert!(decrypted_path.exists());

    //Decrypted file isn't valid
    let decrypted_read_result = fs::read_to_string(decrypted_path.to_str().unwrap());
    assert!(decrypted_read_result.is_err());

    dir.close();
}
