use core::str;

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::generic_array::typenum::Unsigned;
use chacha20poly1305::aead::generic_array::GenericArray;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng};
use chacha20poly1305::ChaCha20Poly1305;

pub fn encrypt(cleartext: &str, key: &[u8]) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let mut obsf = cipher.encrypt(&nonce, cleartext.as_bytes()).unwrap();
    obsf.splice(..0, nonce.iter().copied());

    obsf
}

pub fn decrypt(obsf: &[u8], key: &[u8]) -> Vec<u8> {
    type NonceSize = <ChaCha20Poly1305 as AeadCore>::NonceSize;
    let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(key));
    let (nonce, ciphertext) = obsf.split_at(NonceSize::to_usize());
    let nonce = GenericArray::from_slice(nonce);

    match cipher.decrypt(nonce, ciphertext) {
        Ok(value) => value,
        Err(why) => panic!("{}", why),
    }
}

pub fn hash_password(password: String) -> [u8; 32] {
    let hasher = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).expect("Cannot build hasher params"),
    );

    let mut out = [0u8; 32];
    let salt = [0x02; 16];
    hasher
        .hash_password_into(password.as_bytes(), &salt, &mut out)
        .expect("Failed to hash the password");

    out
}
