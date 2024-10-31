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
struct DownloadOptions {
	pub windows: String,
	pub linux: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RepositoryPlugin {
	pub name: String,
	pub urls: DownloadOptions,
	pub version: String,
	pub state: PluginState,
	pub build_state: BuildState,
}

#[derive(Debug, Deserialize, Serialize)]
struct Repository {
	pub name: String,
	pub plugins: Vec<RepositoryPlugin>,
}

#[derive(Deserialize, Serialize)]
struct PluginNameInternal {
	name: String,
	version: String,
}

pub fn load_repos() -> anyhow::Result<()> {
	for repo_url in &CONFIG.repositories {
		println!("Loading repository: {}", repo_url);
		let repo = reqwest::blocking::get(repo_url).unwrap().json::<Repository>().unwrap();

		if !std::fs::exists(format!("{}/{}", CONFIG.plugins_folder, repo.name))? {
			std::fs::create_dir_all(format!("{}/{}", CONFIG.plugins_folder, repo.name))?;
		}

		let internal_plugins_file = format!("{}/{}/plugins.json", CONFIG.plugins_folder, repo.name);
		let internal_plugins = if std::fs::exists(&internal_plugins_file)? {
			let internal_plugins = std::fs::read_to_string(&internal_plugins_file)?;
			serde_json::from_str::<Vec<PluginNameInternal>>(&internal_plugins)?
		} else {
			let internal_plugins: Vec<PluginNameInternal> = Vec::new();
			std::fs::write(&internal_plugins_file, &serde_json::to_string(&internal_plugins)?)?;
			internal_plugins
		};

		for int_plugin in &internal_plugins {
			if !repo.plugins.iter().any(|p| p.name == int_plugin.name) {
				for ext in PLUGIN_FILE_EXTENSIONS {
					let plugin_file = format!("{}/{}/{}.{}", CONFIG.plugins_folder, repo.name, int_plugin.name, ext);
					if std::fs::exists(&plugin_file)? {
						std::fs::remove_file(&plugin_file)?;
					}
				}
			}
		}

		for plugin in &repo.plugins {
			let internal_plugin = internal_plugins.iter().find(|p| p.name == plugin.name);

			if let Some(internal_plugin) = internal_plugin {
				if internal_plugin.version != plugin.version {
					let plugin_file = if cfg!(target_os = "windows") {
						format!("{}/{}/{}.dll", CONFIG.plugins_folder, repo.name, plugin.name)
					} else {
						format!("{}/{}/{}.so", CONFIG.plugins_folder, repo.name, plugin.name)
					};

					let url = if cfg!(target_os = "windows") {
						&plugin.urls.windows
					} else {
						&plugin.urls.linux
					};

					let plugin_data = reqwest::blocking::get(url).unwrap().bytes().unwrap();
					std::fs::write(&plugin_file, plugin_data).unwrap();
				}
			} else {
				let plugin_file = if cfg!(target_os = "windows") {
					format!("{}/{}/{}.dll", CONFIG.plugins_folder, repo.name, plugin.name)
				} else {
					format!("{}/{}/{}.so", CONFIG.plugins_folder, repo.name, plugin.name)
				};

				let url = if cfg!(target_os = "windows") {
					&plugin.urls.windows
				} else {
					&plugin.urls.linux
				};

				let plugin_data = reqwest::blocking::get(url).unwrap().bytes().unwrap();
				std::fs::write(&plugin_file, plugin_data).unwrap();
			}
		}

		let internal_plugins: Vec<PluginNameInternal> = repo
			.plugins
			.iter()
			.map(|p| PluginNameInternal {
				name: p.name.clone(),
				version: p.version.clone(),
			})
			.collect();
		std::fs::write(&internal_plugins_file, &serde_json::to_string(&internal_plugins)?)?;
	}

	Ok(())
}
