extern crate rand;
extern crate crypto;
extern crate aes;
extern crate base64;

use aes::block_cipher_trait::generic_array::GenericArray;
use aes::block_cipher_trait::BlockCipher;
use aes::Aes128;
use rand::rngs::OsRng;
use rand::RngCore;
use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;


pub enum DecriptionError {
    Unknown,
    VerificationFailure,
    WrongFormat,
    Base64Failure,
}

pub fn generate_key() -> Vec<u8>{
    let mut key = [0u8; 16];
    OsRng.fill_bytes(&mut key);
    return key.to_vec();
}

/// Generates hash for the string. All password must go through this function
fn get_hash(data: &Vec<u8>) -> String {
    let mut hasher = Sha1::new();
    hasher.input(data);
    let buff_str = hasher.result_str();

    buff_str
}

fn encrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let t_key = GenericArray::from_slice(&key[0..16]);
    let mut data_to_encrypt = data.clone();
    data_to_encrypt.resize(((data_to_encrypt.len() as f64 / 16 as f64).ceil() * 16 as f64) as usize, 0);
    let cipher = Aes128::new(&t_key);

    let mut target: Vec<u8> = vec![];

    for x in 0..(data_to_encrypt.len() / 16) {
        let mut block = GenericArray::clone_from_slice(&data_to_encrypt[x * 16..x * 16 + 16]);
        cipher.encrypt_block(&mut block);
        target.extend_from_slice(block.as_slice());
    }

    target
}


fn decrypt(data: &Vec<u8>, key: &Vec<u8>) -> Vec<u8> {
    let t_key = GenericArray::from_slice(&key[0..16]);
    let  data_to_decrypt = data.clone();
    let cipher = Aes128::new(&t_key);

    let mut target: Vec<u8> = vec![];

    for x in 0..(data_to_decrypt.len() / 16) {
        let mut block = GenericArray::clone_from_slice(&data_to_decrypt[x * 16..x * 16 + 16]);
        cipher.decrypt_block(&mut block);
        target.extend_from_slice(block.as_slice());
    }

    target
}


pub fn to_base64(data: &Vec<u8>) -> String {
    base64::encode(data)
}

pub fn from_base64(data: &str) -> Result<Vec<u8>, DecriptionError> {
    base64::decode(data).map_err(|_| {
        DecriptionError::Base64Failure
    })
}


pub fn sign_and_encrypt(data: &Vec<u8>, key: &Vec<u8>) -> String {
    let encrypted = encrypt(data, key);
    let hash = get_hash(data);
    format!("{}==={}", hash, to_base64(&encrypted))
}


pub fn verify_and_decrypt(data: &str, key: &Vec<u8>) -> Result<Vec<u8>, DecriptionError> {
    let parts = data.split("===").map(|x| x.to_string()).collect::<Vec<String>>();
    if parts.len() < 2 || parts[0].len() != 40 || (parts[1].len() / 16) * 16 != parts[1].len() {
        return Err(DecriptionError::WrongFormat);
    }
    let unbased = from_base64(&parts[1])?;
    let decrypted = decrypt(&unbased, key);
    if get_hash(&decrypted) != parts[0] {
        return Err(DecriptionError::VerificationFailure);
    }
    Ok(decrypted)
}