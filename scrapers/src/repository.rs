use config::CONFIG;
use serde::{Deserialize, Serialize};

use crate::PLUGIN_FILE_EXTENSIONS;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BuildState {
	Alpha,
	Beta,
	Stable,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum PluginState {
	Outdated,
	Updated,
	Obsolete,
}

#[derive(Debug, Deserialize, Serialize)]
struct RepositoryPlugin {
	pub name: String,
	pub url: String,
	pub version: String,
	pub state: PluginState,
	pub build_state: BuildState,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repository {
	pub name: String,
	pub url: String,
	pub plugins: Vec<RepositoryPlugin>,
}

struct PluginNameInternal {
	name: String,
	version: String,
}

pub fn load_repos() -> anyhow::Result<()> {
	for repo_url in &CONFIG.repositories {
		let repo = reqwest::blocking::get(repo_url).unwrap().json::<Repository>().unwrap();

		if !std::fs::exists(format!("{}/{}", CONFIG.plugins_folder, repo.name))? {
			std::fs::create_dir_all(format!("{}/{}", CONFIG.plugins_folder, repo.name))?;
		}

		let internal_plugins = std::fs::read_dir(format!("{}/{}", CONFIG.plugins_folder, repo.name))?
			.filter_map(|entry| {
				let entry = entry.ok()?;
				let path = entry.path();
				if path.is_file() {
					if let Some(ext) = path.extension() {
						if PLUGIN_FILE_EXTENSIONS.contains(&ext.to_str().unwrap()) {
							let full_name = path.file_name().unwrap().to_str().unwrap();
							let plugin_name = full_name.split('-').next().unwrap();
							let plugin_version = full_name.split('-').last().unwrap();

							return Some(PluginNameInternal {
								name: plugin_name.to_string(),
								version: plugin_version.to_string(),
							});
						}
					}
				}
				None
			})
			.collect::<Vec<PluginNameInternal>>();

		for plugin in &repo.plugins {
			let internal_plugin = internal_plugins.iter().find(|p| p.name == plugin.name);

			if let Some(internal_plugin) = internal_plugin {
				if internal_plugin.version != plugin.version {
					let plugin_file = format!(
						"{}/{}/{}-{}.so",
						CONFIG.plugins_folder, repo.name, plugin.name, plugin.version
					);
					let plugin_data = reqwest::blocking::get(&plugin.url).unwrap().bytes().unwrap();
					std::fs::write(&plugin_file, plugin_data).unwrap();
				}
			} else {
				let plugin_file = format!(
					"{}/{}/{}-{}.so",
					CONFIG.plugins_folder, repo.name, plugin.name, plugin.version
				);
				let plugin_data = reqwest::blocking::get(&plugin.url).unwrap().bytes().unwrap();
				std::fs::write(&plugin_file, plugin_data).unwrap();
			}
		}
	}

	Ok(())
}
