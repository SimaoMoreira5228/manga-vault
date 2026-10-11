use aes_gcm::aead::consts::U12;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};

const NONCE_LEN: usize = 12;

pub fn validate(secret_key: &str) -> Result<(), String> {
	cipher(secret_key).map(|_| ())
}

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
		.map_err(|_| "cannot decrypt stored credentials: wrong SECRET_KEY or corrupted data".to_string())?;
	String::from_utf8(plaintext).map_err(|error| format!("decrypted key is not utf-8: {error}"))
}

#[cfg(test)]
mod tests {
	use super::*;

	const KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
	const OTHER_KEY: &str = "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";

	#[test]
	fn round_trips_a_credential() {
		let blob = encrypt(KEY, "anilist-token").unwrap();
		assert_eq!(decrypt(KEY, &blob).unwrap(), "anilist-token");
	}

	#[test]
	fn each_encryption_uses_a_fresh_nonce() {
		assert_ne!(encrypt(KEY, "same").unwrap(), encrypt(KEY, "same").unwrap());
	}

	#[test]
	fn a_rotated_key_reports_the_actual_cause() {
		let blob = encrypt(KEY, "anilist-token").unwrap();
		let error = decrypt(OTHER_KEY, &blob).unwrap_err();
		assert!(error.contains("wrong SECRET_KEY"), "unhelpful error: {error}");
	}

	#[test]
	fn truncated_payloads_are_rejected_before_decryption() {
		assert!(decrypt(KEY, &[0u8; NONCE_LEN]).is_err());
	}

	#[test]
	fn validate_accepts_only_thirty_two_byte_keys() {
		assert!(validate(KEY).is_ok());
		assert!(validate("").is_err());
		assert!(validate("not hex").is_err());
		assert!(validate(&"ab".repeat(31)).is_err());
		assert!(validate(&"ab".repeat(33)).is_err());
	}
}
