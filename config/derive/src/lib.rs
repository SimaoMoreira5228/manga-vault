use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, parse_macro_input};

#[proc_macro_derive(Config, attributes(config))]
pub fn derive_config(input: TokenStream) -> TokenStream {
	let ast = parse_macro_input!(input as DeriveInput);
	let name = &ast.ident;

	let mut file_base = name.to_string().trim_end_matches("Config").to_lowercase();

	for attr in &ast.attrs {
		if !attr.path().is_ident("config") {
			continue;
		}
		let mut found = false;
		let _ = attr.parse_nested_meta(|meta| {
			if meta.path.is_ident("name") {
				if let Ok(Lit::Str(lit)) = meta.value().and_then(|v| v.parse()) {
					file_base = lit.value();
					found = true;
				}
			}
			Ok(())
		});
		if found {
			break;
		}
	}

	let base_str = file_base.clone();

	let expanded = quote! {
		impl #name {
			pub fn load() -> Self {
				let base_path = std::env::current_exe()
					.expect("Failed to get executable path")
					.parent()
					.expect("Executable has no parent directory")
					.to_path_buf();
				let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
				let config_path = format!("{}/config/{}", base_path.display(), #base_str);
				config::load_config::<Self>(&config_path, &env)
					.expect("Failed to load config")
			}

			pub fn dump_schema() {
				let base_path = std::env::current_exe()
					.expect("Failed to get executable path")
					.parent()
					.expect("Executable has no parent directory")
					.to_path_buf();
				let config_dir = base_path.join("config");
				let file_name = format!("{}.toml", #base_str);
				let path = config_dir.join(file_name);

				let default = Self::default();
				let toml = toml::to_string_pretty(&default)
					.expect("Failed to serialize default config");
				std::fs::create_dir_all(&config_dir).ok();
				std::fs::write(&path, toml)
					.expect("Failed to write default config file");
				println!("Created default config: {}", path.display());
			}
		}
	};

	TokenStream::from(expanded)
}
