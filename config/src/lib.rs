#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

pub fn load_config<T>(base: &str, env_name: &str) -> Result<T, Box<dyn std::error::Error>>
where
	T: DeserializeOwned + Default + Serialize,
{
	let main_file = format!("{base}.json");
	let env_file = format!("{base}.{env_name}.json");
	let config_dir = Path::new(base).parent().unwrap_or_else(|| Path::new("config"));

	let default_config = T::default();

	if !Path::new(&main_file).exists() {
		fs::create_dir_all(config_dir)?;
		let default = T::default();
		let toml = serde_json::to_string_pretty(&default)?;
		fs::write(&main_file, toml)?;
		eprintln!("⚠️  Created default config: {}", main_file);
	}

	let config = read_json_file(&main_file).unwrap_or_else(|_| Value::Object(Map::new()));
	let default_value = serde_json::to_value(&default_config)?;

	let mut merged_config = default_value.clone();
	merge_values(&mut merged_config, config);

	if let Ok(env_config) = read_json_file(&env_file) {
		merge_values(&mut merged_config, env_config);
	}

	let env_vars = read_env_vars("MVAULT", "__");
	merge_values(&mut merged_config, env_vars);

	let updated_json = serde_json::to_string_pretty(&merged_config)?;
	fs::write(&main_file, updated_json)?;

	let result: T = serde_json::from_value(merged_config)?;
	Ok(result)
}

fn read_json_file(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
	let mut file = File::open(path)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	Ok(serde_json::from_str(&contents)?)
}

fn read_env_vars(prefix: &str, separator: &str) -> Value {
	let mut root = Map::new();
	let full_prefix = format!("{}{}", prefix, separator);

	for (key, value) in env::vars() {
		if key.starts_with(&full_prefix) {
			let path = key[full_prefix.len()..].split(separator).collect::<Vec<&str>>();

			build_value_tree(&mut root, &path, Value::String(value));
		}
	}
	Value::Object(root)
}

fn build_value_tree(map: &mut Map<String, Value>, path: &[&str], value: Value) {
	if path.is_empty() {
		return;
	}

	let (current, rest) = path.split_first().unwrap();
	if rest.is_empty() {
		map.insert(current.to_string(), value);
	} else {
		let entry = map.entry(current.to_string()).or_insert_with(|| Value::Object(Map::new()));
		if let Value::Object(nested) = entry {
			build_value_tree(nested, rest, value);
		}
	}
}

fn merge_values(a: &mut Value, b: Value) {
	if let (Value::Object(a), Value::Object(b)) = (a, b) {
		for (key, value) in b {
			if let Some(existing) = a.get_mut(&key) {
				if existing.is_object() && value.is_object() {
					merge_values(existing, value);
					continue;
				}
			}
			a.insert(key, value);
		}
	}
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
	use super::*;

	#[test]
	fn test_build_value_tree() {
		let mut map = Map::new();
		let path = vec!["a", "b", "c"];
		let value = Value::String("test".to_string());

		build_value_tree(&mut map, &path, value.clone());
		assert_eq!(map.get("a").unwrap().get("b").unwrap().get("c").unwrap(), &value);
	}

	#[test]
	fn test_merge_values() {
		let mut a = Value::Object(Map::new());
		let b = Value::Object(Map::from_iter([
			("x".to_string(), Value::String("1".to_string())),
			(
				"y".to_string(),
				Value::Object(Map::from_iter([("z".to_string(), Value::String("2".to_string()))])),
			),
		]));
		merge_values(&mut a, b);
		assert_eq!(
			a,
			Value::Object(Map::from_iter([
				("x".to_string(), Value::String("1".to_string())),
				(
					"y".to_string(),
					Value::Object(Map::from_iter([("z".to_string(), Value::String("2".to_string()))]))
				),
			]))
		);
	}
}
