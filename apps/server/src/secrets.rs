use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::consts::U12;

const NONCE_LEN: usize = 12;

fn cipher(secret_key: &str) -> Result<Aes256Gcm, String> {
	let bytes = hex_decode(secret_key).ok_or("SECRET_KEY must be 64 hex chars (32 bytes)")?;
	if bytes.len() != 32 {
		return Err("SECRET_KEY must be 64 hex chars (32 bytes)".into());
	}
	Ok(Aes256Gcm::new_from_slice(&bytes).expect("valid key length"))
}

fn random_nonce() -> Nonce<U12> {
	let bytes: [u8; NONCE_LEN] = rand::random();
	bytes.into()
}

fn hex_decode(value: &str) -> Option<Vec<u8>> {
	if !value.len().is_multiple_of(2) {
		return None;
	}
	(0..value.len())
		.step_by(2)
		.map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
		.collect()
}

pub fn encrypt(secret_key: &str, plaintext: &str) -> Result<Vec<u8>, String> {
	let cipher = cipher(secret_key)?;
	let nonce = random_nonce();
	let ciphertext = cipher
		.encrypt(&nonce, plaintext.as_bytes())
		.map_err(|error| format!("encryption failed: {error}"))?;
	let mut blob = nonce.to_vec();
	blob.extend(ciphertext);
	Ok(blob)
}

pub fn decrypt(secret_key: &str, blob: &[u8]) -> Result<String, String> {
	if blob.len() <= NONCE_LEN {
		return Err("encrypted payload is too short".into());
	}
	let cipher = cipher(secret_key)?;
	let (nonce, ciphertext) = blob.split_at(NONCE_LEN);
	let nonce = <&Nonce<U12>>::try_from(nonce).map_err(|_| "invalid nonce".to_string())?;
	let plaintext = cipher
		.decrypt(nonce, ciphertext)
		.map_err(|error| format!("decryption failed: {error}"))?;
	String::from_utf8(plaintext).map_err(|error| format!("decrypted key is not utf-8: {error}"))
}
