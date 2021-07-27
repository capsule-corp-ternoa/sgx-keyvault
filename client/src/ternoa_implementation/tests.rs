use crate::ternoa_implementation::cipher::{
    aes256key_from_shamir_shares, decrypt_with_key, encrypt, recover_or_generate_encryption_key,
    shamir_shares_from_file, Key,
};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use sharks::{Share, Sharks};
use tempfile::tempdir;

const KEYFILE_EXT: &str = "aes256";
const CIPHERTEXT_EXT: &str = "ciphertext";
const DECRYPTED_EXT: &str = "decrypted";
const TEST_TEXT: &str = "I'm nobody! Who are you?\nAre you nobody, too?";
const THRESHOLD_SHAMIR_SHARE: u8 = 8;
const NUM_SHAMIR_SHARE_SPLIT: usize = 12;

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

fn aes256_key() -> Key {
    let key: Vec<u8> = vec![
        69, 251, 240, 219, 35, 251, 0, 132, 240, 231, 99, 216, 223, 63, 75, 46, 51, 111, 168, 250,
        192, 23, 84, 185, 231, 224, 178, 161, 27, 60, 194, 65,
    ];
    let iv: Vec<u8> = vec![
        102, 131, 19, 249, 75, 147, 95, 165, 216, 140, 106, 71, 126, 19, 57, 35,
    ];
    (key, iv)
}

/// shamir split aes256key into NUM_SHAMIR_SHARE_SPLIT shares, of which THRESHOLD_SHAMIR_SHARE are needed for key recovery
fn shamir_shares(aes256_key: Key, num_shares_saved: usize) -> Vec<Share> {
    // Set a minimum threshold of n shares
    let sharks = Sharks(THRESHOLD_SHAMIR_SHARE);

    // Obtain an iterator over the shares for secret
    let mut secret = aes256_key.0;
    secret.extend(aes256_key.1);
    let dealer = sharks.dealer(&secret);
    let mut shares: Vec<Share> = dealer.take(NUM_SHAMIR_SHARE_SPLIT).collect();
    shares.truncate(num_shares_saved);
    shares
}

#[test]
fn verify_recover_encryption_key() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //When
    let key = recover_or_generate_encryption_key(&key_path).unwrap();
    let recovered_key = recover_or_generate_encryption_key(&key_path).unwrap();
    //Then
    assert!(key_path.exists());
    assert_eq!(recovered_key.0, key.0);
    assert_eq!(recovered_key.1, key.1);
    //Clean
    dir.close();
}

#[test]
fn verify_generate_encryption_key() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //When
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    //Then
    assert_eq!(aes.0.len(), 32);
    assert_eq!(aes.1.len(), 16);
    assert!(key_path.exists());

    dir.close();
}

#[test]
fn verify_encrypt_generate_key_when_no_key_passed() {
    //Given
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("input.".to_owned() + KEYFILE_EXT);
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();

    //When
    encrypt(inputfile_path.to_str().unwrap(), None).unwrap();

    // Then
    assert!(key_path.exists()); //A key has been generated
    dir.close();
}

#[test]
fn verify_encrypt_without_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();

    //When
    let result = encrypt(inputfile_path.to_str().unwrap(), None);

    // Then
    assert!(result.is_ok());
    assert!(ciphertext_path.exists());
    dir.close();
}

#[test]
fn verify_decrypt_without_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();
    encrypt(inputfile_path.to_str().unwrap(), None).unwrap();

    //When
    let result = decrypt_with_key(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_ok());
    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_encrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    //generate key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();

    //When
    let result = encrypt(test_file_path.to_str().unwrap(), Some(aes.clone()));
    //Then
    assert!(result.is_ok());
    assert!(ciphertext_path.exists());

    dir.close();
}

#[test]
fn verify_decrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();
    //generate key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    //encrypt
    encrypt(test_file_path.to_str().unwrap(), Some(aes.clone())).unwrap();

    //When
    let result = decrypt_with_key(ciphertext_path.to_str().unwrap(), Some(aes));

    //Then
    assert!(result.is_ok());
    assert!(decrypted_path.exists());
    //Check content
    let text = decrypted_text(decrypted_path.to_str().unwrap());
    assert_eq!(text, TEST_TEXT);

    dir.close();
}

#[test]
fn verify_decrypt_fails_without_keyfile() {
    //Given
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("tmp");
    let mut test_file = File::create(file_path.clone()).unwrap();
    writeln!(test_file, "blablabla").unwrap();

    let ciphertext_path = file_path.join("tmp.").join(CIPHERTEXT_EXT);

    encrypt(file_path.to_str().unwrap(), None).unwrap();

    let key_path = file_path.join("tmp.").join(KEYFILE_EXT);
    fs::remove_file(key_path);

    //When
    let result = decrypt_with_key(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_err());
    dir.close();
}

#[test]
fn verify_decrypt_fails_with_diff_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let decrypted_path = dir.path().join("input.".to_owned() + DECRYPTED_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    let other_key_path = dir.path().join("keyfile2.".to_owned() + KEYFILE_EXT);
    let test_file_path = plaintext_input(dir.path()).ok().unwrap();

    //encrypt with one key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    encrypt(test_file_path.to_str().unwrap(), Some(aes)).unwrap();

    //When
    //decrypt with another key
    let other_aes = recover_or_generate_encryption_key(&other_key_path).unwrap();
    decrypt_with_key(ciphertext_path.to_str().unwrap(), Some(other_aes)).unwrap();

    //Then
    assert!(decrypted_path.exists());
    //Decrypted file isn't valid
    let decrypted_read_result = fs::read_to_string(decrypted_path.to_str().unwrap());
    assert!(decrypted_read_result.is_err());

    dir.close();
}

#[test]
fn verify_decrypt_without_passing_key_fails_when_encrypt_by_passing_key() {
    //Given
    let dir = tempdir().unwrap();
    let ciphertext_path = dir.path().join("input.".to_owned() + CIPHERTEXT_EXT);
    let key_path = dir.path().join("keyfile.".to_owned() + KEYFILE_EXT);
    //Create test input file
    let inputfile_path = plaintext_input(dir.path()).ok().unwrap();
    //encrypt with one key
    let aes = recover_or_generate_encryption_key(&key_path).unwrap();
    encrypt(inputfile_path.to_str().unwrap(), Some(aes)).unwrap();

    //When
    let result = decrypt_with_key(ciphertext_path.to_str().unwrap(), None);

    //Then
    assert!(result.is_err());
    dir.close();
}

#[test]
fn verify_recover_key_from_shamir_with_threshold_number_shares() {
    // given
    let aes256_key: Key = aes256_key();
    let shares = shamir_shares(aes256_key.clone(), THRESHOLD_SHAMIR_SHARE as usize);

    // when
    let encryption_key = aes256key_from_shamir_shares(shares).unwrap();

    // then
    assert_eq!(aes256_key.0, encryption_key.0);
    assert_eq!(aes256_key.1, encryption_key.1);
}

#[test]
fn verify_recover_key_from_shamir_with_more_shares_than_threshold() {
    // given
    let aes256_key: Key = aes256_key();
    let shares = shamir_shares(aes256_key.clone(), (THRESHOLD_SHAMIR_SHARE + 1) as usize);

    // when
    let encryption_key = aes256key_from_shamir_shares(shares).unwrap();

    //then
    assert_eq!(aes256_key.0, encryption_key.0);
    assert_eq!(aes256_key.1, encryption_key.1);
}

#[test]
fn verify_recover_wrong_key_from_shamir_with_less_shares_than_threshold() {
    // given
    let aes256_key: Key = aes256_key();
    let shares = shamir_shares(aes256_key.clone(), (THRESHOLD_SHAMIR_SHARE - 1) as usize);

    // when
    let encryption_key = aes256key_from_shamir_shares(shares).unwrap();

    // then
    assert_ne!(aes256_key.0, encryption_key.0);
    assert_ne!(aes256_key.1, encryption_key.1);
}

#[test]
fn verify_recover_key_from_shamir_share_file() {
    // given
    let path = "test_key_shamir_file";
    let filename = "shamir_shares.txt";
    let file_path = PathBuf::from(path).join(filename);
    //smaller file generated with threshold 2 and shares number 5 -> 3 shares enough
    let threshold = 2u8;

    let key: Vec<u8> = vec![
        215, 50, 27, 231, 182, 159, 175, 45, 147, 110, 99, 107, 133, 69, 225, 112, 5, 131, 148, 83,
        247, 123, 245, 81, 34, 239, 224, 91, 185, 117, 82, 204,
    ];
    let iv: Vec<u8> = vec![
        166, 158, 168, 213, 158, 174, 158, 77, 178, 116, 183, 204, 176, 42, 95, 120,
    ];
    let aes256_key: Key = (key, iv);

    //shamir shares file
    let line1 ="01a6198ad98e7e7daf2482dee279dacfdb840ee7e73d8e95f216f03c7a62c192513dfd927dc6e2388517427a91b7ff8583";
    let line2 = "023564249bc6401634e0ab04646066bd3b1a8472267e8c350a4ad145191200cfeb8d58dc982e36cfc0e5183076be9df693";
    let line3 ="03444fb5a5fea1c4b65747b9ed9cf993909b090192b47955a97ece9938c9b40f76163be630767a6908402efd2bb9482c68";
    let text = format! {"{}\n{}\n{}", line1, line2, line3};
    // create file
    fs::create_dir_all(path).unwrap();
    let mut file = fs::File::create(file_path.clone()).unwrap();
    file.write_all(text.as_bytes()).unwrap();
    let shares = shamir_shares_from_file(file_path).unwrap();

    // when
    let encryption_key = aes256key_from_shamir_shares(shares).unwrap();

    // then
    assert_eq!(aes256_key.0, encryption_key.0);
    assert_eq!(aes256_key.1, encryption_key.1);
    //clean up
    fs::remove_dir_all(path).unwrap();
}

#[test]
fn verify_recover_key_fails_with_shamir_share_file_empty() {
    // given
    let path = "test_key_from_shamir_empty";
    let filename = "empty_file.txt";
    let file_path = PathBuf::from(path).join(filename);

    //shamir shares empty file
    fs::create_dir_all(path).unwrap();
    let mut file = fs::File::create(file_path.clone()).unwrap();
    file.write_all("".as_bytes()).unwrap();

    // when
    let shares = shamir_shares_from_file(file_path).unwrap();

    // then
    assert!(aes256key_from_shamir_shares(shares).is_err());

    //clean up
    fs::remove_dir_all(path).unwrap();
}
#[test]
fn verify_recover_diff_key_with_wrong_shamir_share_file() {
    // given
    let aes256_key: Key = aes256_key();

    let path = "test_key_from_wrong_shamir";
    let filename = "shamir_shares.txt";
    let file_path = PathBuf::from(path).join(filename);

    //shamir shares file from other key
    let line1 ="01a6198ad98e7e7daf2482dee279dacfdb840ee7e73d8e95f216f03c7a62c192513dfd927dc6e2388517427a91b7ff8583";
    let line2 = "023564249bc6401634e0ab04646066bd3b1a8472267e8c350a4ad145191200cfeb8d58dc982e36cfc0e5183076be9df693";
    let line3 ="03444fb5a5fea1c4b65747b9ed9cf993909b090192b47955a97ece9938c9b40f76163be630767a6908402efd2bb9482c68";
    let text = format! {"{}\n{}\n{}", line1, line2, line3};
    // create file
    fs::create_dir_all(path).unwrap();
    let mut file = fs::File::create(file_path.clone()).unwrap();
    file.write_all(text.as_bytes()).unwrap();
    let shares = shamir_shares_from_file(file_path).unwrap();

    // when
    let encryption_key = aes256key_from_shamir_shares(shares).unwrap();

    // then
    assert_ne!(aes256_key.0, encryption_key.0);
    assert_ne!(aes256_key.1, encryption_key.1);

    //clean up
    fs::remove_dir_all(path).unwrap();
}
