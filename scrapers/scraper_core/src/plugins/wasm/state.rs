use wasmtime::component::ResourceTable;
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::plugins::wasm::bindings;

pub struct States {
	table: ResourceTable,
	ctx: WasiCtx,
}

impl States {
	pub fn new() -> Self {
		let table = ResourceTable::new();
		let ctx = WasiCtxBuilder::new().build();
		Self { table, ctx }
	}
}

impl wasmtime_wasi::p2::IoView for States {
	fn table(&mut self) -> &mut ResourceTable {
		&mut self.table
	}
}

impl WasiView for States {
	fn ctx(&mut self) -> &mut WasiCtx {
		&mut self.ctx
	}
}

impl bindings::scraper::types::http::Host for States {
	fn get(
		&mut self,
		url: String,
		headers: Option<Vec<bindings::scraper::types::http::Header>>,
	) -> Option<bindings::scraper::types::http::Response> {
		let headers = headers.unwrap_or_default();
		let client = reqwest::blocking::Client::new();
		let mut request = client.get(&url);
		for header in headers {
			request = request.header(header.name, header.value);
		}

		match request.send() {
			Ok(res) => {
				let status = res.status().as_u16() as u32;
				let headers = res
					.headers()
					.iter()
					.map(|(name, value)| bindings::scraper::types::http::Header {
						name: name.to_string(),
						value: value.to_str().unwrap_or("").to_string(),
					})
					.collect();
				let body = res.text().unwrap_or_default();
				Some(bindings::scraper::types::http::Response { status, headers, body })
			}
			Err(e) => {
				eprintln!("Error fetching URL {}: {}", url, e);
				None
			}
		}
	}

	fn post(
		&mut self,
		url: String,
		body: String,
		headers: Option<Vec<bindings::scraper::types::http::Header>>,
	) -> Option<bindings::scraper::types::http::Response> {
		let headers = headers.unwrap_or_default();
		let client = reqwest::blocking::Client::new();
		let mut request = client.post(&url).body(body);
		for header in headers {
			request = request.header(header.name, header.value);
		}

		match request.send() {
			Ok(res) => {
				let status = res.status().as_u16() as u32;
				let headers = res
					.headers()
					.iter()
					.map(|(name, value)| bindings::scraper::types::http::Header {
						name: name.to_string(),
						value: value.to_str().unwrap_or("").to_string(),
					})
					.collect();
				let body = res.text().unwrap_or_default();
				Some(bindings::scraper::types::http::Response { status, headers, body })
			}
			Err(e) => {
				eprintln!("Error posting to URL {}: {}", url, e);
				None
			}
		}
	}
}
