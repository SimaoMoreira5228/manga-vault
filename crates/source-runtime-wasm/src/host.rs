use std::collections::HashMap;

use source_sdk::cloudflare::looks_like_cloudflare_protection;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_io::IoView;

use crate::bindings::manga_vault::source::{flare_solverr, html, http, types};

pub struct WasmState {
	pub table: wasmtime::component::ResourceTable,
	pub wasi: WasiCtx,
	pub client: reqwest::Client,
	pub flaresolverr_url: Option<String>,
}

impl IoView for WasmState {
	fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
		&mut self.table
	}
}

impl WasiView for WasmState {
	fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
		wasmtime_wasi::WasiCtxView {
			ctx: &mut self.wasi,
			table: &mut self.table,
		}
	}
}

impl WasmState {
	pub fn new(flaresolverr_url: Option<String>) -> Self {
		let client = reqwest::Client::builder()
			.timeout(std::time::Duration::from_secs(30))
			.user_agent(source_sdk::BROWSER_USER_AGENT)
			.build()
			.expect("reqwest client");
		Self {
			wasi: WasiCtxBuilder::new().inherit_stdio().build(),
			table: Default::default(),
			client,
			flaresolverr_url,
		}
	}
}

fn header_list(headers: &reqwest::header::HeaderMap) -> Vec<http::Header> {
	headers
		.iter()
		.map(|(k, v)| http::Header {
			name: k.as_str().to_owned(),
			value: v.to_str().unwrap_or_default().to_owned(),
		})
		.collect()
}

async fn fetch(_client: &reqwest::Client, request: reqwest::RequestBuilder) -> Result<Option<http::Response>, String> {
	let Ok(response) = request.send().await else { return Ok(None) };
	let status = response.status().as_u16();
	let headers = header_list(response.headers());
	let body = response.text().await.unwrap_or_default();
	Ok(Some(http::Response { status, headers, body }))
}

impl http::Host for WasmState {
	async fn get(&mut self, url: String, headers: Option<Vec<http::Header>>) -> Result<Option<http::Response>, String> {
		let mut builder = self.client.get(&url);
		for header in headers.into_iter().flatten() {
			builder = builder.header(header.name, header.value);
		}
		fetch(&self.client, builder).await
	}

	async fn post(
		&mut self,
		url: String,
		body: String,
		headers: Option<Vec<http::Header>>,
	) -> Result<Option<http::Response>, String> {
		let mut builder = self.client.post(&url);
		for header in headers.into_iter().flatten() {
			builder = builder.header(header.name, header.value);
		}
		fetch(&self.client, builder.body(body)).await
	}

	async fn has_cloudflare_protection(
		&mut self,
		text: String,
		status_code: Option<u16>,
		headers: Option<Vec<http::Header>>,
	) -> bool {
		let list: Vec<(String, String)> = headers.into_iter().flatten().map(|h| (h.name, h.value)).collect();
		looks_like_cloudflare_protection(&text, status_code, &list)
	}
}

impl flare_solverr::Host for WasmState {
	async fn get(&mut self, url: String, session_id: Option<String>) -> Result<Option<http::Response>, String> {
		let Some(endpoint) = self.flaresolverr_url.clone() else {
			return Err("flaresolverr is not configured on this instance".into());
		};
		let mut payload = serde_json::json!({ "cmd": "request.get", "url": url, "maxTimeout": 60_000 });
		if let Some(session) = session_id {
			payload["session"] = serde_json::Value::String(session);
		}
		let Ok(response) = self.client.post(endpoint).json(&payload).send().await else {
			return Ok(None);
		};
		let Ok(json) = response.json::<serde_json::Value>().await else {
			return Ok(None);
		};
		let Some(solution) = json.get("solution") else {
			return Ok(None);
		};
		let headers: HashMap<String, String> = solution
			.get("headers")
			.and_then(|h| serde_json::from_value(h.clone()).ok())
			.unwrap_or_default();
		Ok(Some(http::Response {
			status: solution.get("status").and_then(|s| s.as_u64()).unwrap_or_default() as u16,
			headers: headers
				.into_iter()
				.map(|(name, value)| http::Header { name, value })
				.collect(),
			body: solution
				.get("response")
				.and_then(|b| b.as_str())
				.unwrap_or_default()
				.to_owned(),
		}))
	}
}

impl html::Host for WasmState {
	async fn find(&mut self, document: String, selector: String) -> Vec<html::Element> {
		source_sdk::selection::select_all(&document, &selector)
			.into_iter()
			.map(|html| html::Element {
				html,
				selector: selector.clone(),
			})
			.collect()
	}

	async fn find_one(&mut self, document: String, selector: String) -> Option<html::Element> {
		source_sdk::selection::select_all(&document, &selector)
			.into_iter()
			.next()
			.map(|html| html::Element { html, selector })
	}

	async fn text(&mut self, elem: html::Element) -> String {
		source_sdk::selection::fragment_text(&elem.html)
	}

	async fn attr(&mut self, elem: html::Element, name: String) -> Option<String> {
		source_sdk::selection::fragment_attr(&elem.html, &name)
	}
}

impl types::Host for WasmState {}
