use base64::{Engine as _, engine::general_purpose};
use md5;
use mlua::{IntoLua, Lua, LuaSerdeExt, Table};
use openssl::symm::{Cipher, decrypt};
use scraper_types::{ScraperError, ScraperErrorKind};

use crate::plugins::common::http::Response;

pub fn create_response_table(lua: &Lua, response: Response) -> mlua::Result<Table> {
	let response_table = lua.create_table()?;

	let text = response.text.clone();
	response_table.set("text", response.text)?;
	response_table.set("status", response.status)?;
	response_table.set("ok", response.ok)?;

	let headers_table = lua.create_table()?;
	for (key, value) in response.headers {
		headers_table.set(key, value)?;
	}
	response_table.set("headers", headers_table)?;

	if let Some(error) = response.error {
		response_table.set("error", error.into_lua(lua)?)?;
	} else {
		response_table.set("error", mlua::Value::Nil)?;
	}

	response_table.set(
		"json",
		lua.create_function(move |lua, ()| {
			let json: serde_json::Value = match serde_json::from_str(&text) {
				Ok(value) => value,
				Err(_) => serde_json::Value::Null,
			};
			lua.to_value(&json)
		})?,
	)?;

	Ok(response_table)
}

pub fn load(lua: &Lua) -> anyhow::Result<()> {
	let utils_table = lua.create_table()?;

	utils_table.set(
		"sleep",
		lua.create_async_function(|_lua, ms: u64| async move {
			tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
			Ok(())
		})?,
	)?;

	utils_table.set(
		"raise_error",
		lua.create_function(
			|_, (kind_str, message, retryable): (String, String, Option<bool>)| -> mlua::Result<()> {
				let kind = ScraperErrorKind::from_str(&kind_str);
				let mut error = ScraperError::new(kind, message);
				if let Some(r) = retryable {
					error.retryable = r;
				}
				Err(mlua::Error::external(error))
			},
		)?,
	)?;

	utils_table.set(
		"base64_decode",
		lua.create_function(|lua, s: String| {
			let decoded = general_purpose::STANDARD.decode(s).map_err(mlua::Error::external)?;
			let s = String::from_utf8(decoded).map_err(mlua::Error::external)?;
			lua.to_value(&s)
		})?,
	)?;

	utils_table.set(
		"base64_encode",
		lua.create_function(|lua, s: String| {
			let encoded = general_purpose::STANDARD.encode(s.as_bytes());
			lua.to_value(&encoded)
		})?,
	)?;

	utils_table.set(
		"aes_decrypt",
		lua.create_function(|lua, (ct_b64, password): (String, String)| {
			let data = general_purpose::STANDARD.decode(ct_b64).map_err(mlua::Error::external)?;
			let (salt_opt, ciphertext) = if data.len() > 16 && &data[0..8] == b"Salted__" {
				(Some(&data[8..16]), &data[16..])
			} else {
				(None, &data[..])
			};

			let key_len = 32;
			let iv_len = 16;
			let mut m = Vec::new();
			let mut prev: Vec<u8> = Vec::new();
			while m.len() < (key_len + iv_len) {
				let mut input = Vec::new();
				if !prev.is_empty() {
					input.extend_from_slice(&prev);
				}
				input.extend_from_slice(password.as_bytes());
				if let Some(salt) = salt_opt {
					input.extend_from_slice(salt);
				}
				let digest = md5::compute(&input);
				prev = digest.0.to_vec();
				m.extend_from_slice(&prev);
			}
			let key = &m[0..key_len];
			let iv = &m[key_len..key_len + iv_len];

			let plain = decrypt(Cipher::aes_256_cbc(), key, Some(iv), ciphertext).map_err(mlua::Error::external)?;
			let s = String::from_utf8(plain).map_err(mlua::Error::external)?;
			lua.to_value(&s)
		})?,
	)?;

	lua.globals().set("utils", utils_table)?;
	Ok(())
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use std::collections::HashMap;
	use std::time::Instant;

	use mlua::Lua;
	use scraper_types::{ScraperError, ScraperErrorKind};

	use crate::plugins::common::http::Response;

	#[tokio::test]
	async fn test_utils_sleep() {
		let lua = Lua::new();
		super::load(&lua).unwrap();
		let script = r#"
			utils.sleep(2000)
			return true
		"#;
		let start = Instant::now();
		let result: bool = lua.load(script).eval_async().await.unwrap();
		let elapsed = start.elapsed();
		assert!(result);
		assert!(elapsed.as_secs_f64() >= 2.0, "Sleep was less than 2 seconds: {:?}", elapsed);
	}

	#[test]
	fn test_response_table_success() {
		let lua = Lua::new();
		let response = Response::success("Hello".to_string(), 200, HashMap::new());

		let table = super::create_response_table(&lua, response).unwrap();

		assert_eq!(table.get::<String>("text").unwrap(), "Hello");
		assert_eq!(table.get::<u16>("status").unwrap(), 200);
		assert!(table.get::<bool>("ok").unwrap());
		assert!(table.get::<mlua::Value>("error").unwrap().is_nil());
	}

	#[test]
	fn test_response_table_error() {
		let lua = Lua::new();
		let error = ScraperError::network("Connection failed");
		let response = Response::from_error(error);

		let table = super::create_response_table(&lua, response).unwrap();

		assert_eq!(table.get::<String>("text").unwrap(), "");
		assert_eq!(table.get::<u16>("status").unwrap(), 0);
		assert!(!table.get::<bool>("ok").unwrap());

		let error_table: mlua::Table = table.get("error").unwrap();
		assert_eq!(error_table.get::<String>("kind").unwrap(), "network");
		assert!(error_table.get::<bool>("retryable").unwrap());
	}

	#[test]
	fn test_raise_error() {
		let lua = Lua::new();
		super::load(&lua).unwrap();

		let script = r#"
            utils.raise_error("network", "Failed manually", true)
        "#;

		let result = lua.load(script).exec();
		match result {
			Err(mlua::Error::CallbackError { cause, .. }) => {
				let err = cause.downcast_ref::<ScraperError>().unwrap();
				assert_eq!(err.kind, ScraperErrorKind::Network);
				assert_eq!(err.message, "Failed manually");
				assert!(err.retryable);
			}
			Err(mlua::Error::ExternalError(cause)) => {
				let err = cause.downcast_ref::<ScraperError>().unwrap();
				assert_eq!(err.kind, ScraperErrorKind::Network);
				assert_eq!(err.message, "Failed manually");
				assert!(err.retryable);
			}
			_ => panic!("Expected callback ScraperError, got {:?}", result),
		}
	}
}
