use std::future::Future;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, bail};
use reqwest::header::{HeaderValue, REFERER};
use source_runtime_lua::LuaRuntime;
use source_runtime_wasm::WasmRuntime;
use source_sdk::{Backend, PluginManifest, Source};

const WASM_TARGET: &str = "wasm32-wasip2";
const IMAGE_USER_AGENT: &str = source_sdk::BROWSER_USER_AGENT;

struct Args {
	command: String,
	dir: PathBuf,
	flaresolverr: Option<String>,
	only: Option<String>,
	query: String,
	work_url: Option<String>,
	download_images: Option<PathBuf>,
}

fn parse_args() -> anyhow::Result<Args> {
	let mut args = std::env::args().skip(1);
	let command = args.next().unwrap_or_else(|| usage());
	let dir = PathBuf::from(args.next().unwrap_or_else(|| usage()));
	let mut parsed = Args {
		command,
		dir,
		flaresolverr: None,
		only: None,
		query: "solo leveling".to_string(),
		work_url: None,
		download_images: None,
	};
	while let Some(arg) = args.next() {
		match arg.as_str() {
			"--flaresolverr" => parsed.flaresolverr = Some(args.next().expect("--flaresolverr needs a value")),
			"--only" => parsed.only = Some(args.next().expect("--only needs a value")),
			"--query" => parsed.query = args.next().expect("--query needs a value"),
			"--url" => parsed.work_url = Some(args.next().expect("--url needs a value")),
			"--download-images" => {
				parsed.download_images = Some(PathBuf::from(args.next().expect("--download-images needs a value")))
			}
			other => bail!("unknown argument: {other}"),
		}
	}

	Ok(parsed)
}

fn usage() -> ! {
	eprintln!(
		"usage: plugin-tester <smoke|tests> <plugin-dir> [--only <step>] [--query <text>] [--url <work-url>] [--download-images <dir>] [--flaresolverr <url>]\n\
		 smoke   live probe: info, latest, trending, search, work details, chapter content\n\
		 tests   lua plugins: run the declared Tests table"
	);
	std::process::exit(2);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = parse_args()?;

	if !args.dir.is_dir() {
		bail!("plugin directory {} does not exist", args.dir.display());
	}

	match args.command.as_str() {
		"smoke" => smoke(&args).await,
		"tests" => declared_tests(&args).await,
		other => bail!("unknown command `{other}` (expected smoke or tests)"),
	}
}

fn manifest_of(dir: &Path) -> anyhow::Result<PluginManifest> {
	PluginManifest::load(dir).map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn backend_of(dir: &Path) -> anyhow::Result<Backend> {
	Ok(manifest_of(dir)?.backend)
}

async fn load_plugin(args: &Args) -> anyhow::Result<Box<dyn Source>> {
	if backend_of(&args.dir)? == Backend::Wasm {
		ensure_wasm_built(&args.dir)?;
	}
	match backend_of(&args.dir)? {
		Backend::Lua => {
			let runtime = LuaRuntime::new(args.flaresolverr.clone());
			Ok(Box::new(runtime.load(&args.dir)?))
		}
		Backend::Wasm => {
			let runtime = WasmRuntime::new(args.flaresolverr.clone())?;
			Ok(Box::new(runtime.load(&args.dir).await?))
		}
	}
}

fn ensure_wasm_built(dir: &Path) -> anyhow::Result<()> {
	let manifest = manifest_of(dir)?;
	let entrypoint = dir.join(&manifest.entrypoint);
	let source = dir.join("src/lib.rs");
	if entrypoint.is_file() && entrypoint.metadata()?.modified()? >= source.metadata()?.modified()? {
		return Ok(());
	}

	println!("building wasm component ({WASM_TARGET})...");
	let status = Command::new("cargo")
		.args(["build", "--target", WASM_TARGET, "--release"])
		.current_dir(dir)
		.status()
		.context("failed to spawn cargo")?;
	assert!(status.success(), "cargo build failed");

	let package = package_name(dir)?;
	let built = dir
		.join("target")
		.join(WASM_TARGET)
		.join("release")
		.join(format!("{package}.wasm"));
	std::fs::copy(&built, &entrypoint).with_context(|| format!("copying {} into bundle", built.display()))?;
	println!("built {}", manifest.entrypoint);
	Ok(())
}

fn package_name(dir: &Path) -> anyhow::Result<String> {
	let text = std::fs::read_to_string(dir.join("Cargo.toml")).context("missing Cargo.toml")?;
	text.lines()
		.find_map(|line| line.trim_start().strip_prefix("name = \""))
		.and_then(|rest| rest.split('"').next())
		.map(|name| name.replace('-', "_"))
		.context("malformed package name")
}

async fn declared_tests(args: &Args) -> anyhow::Result<()> {
	let runtime = LuaRuntime::new(args.flaresolverr.clone());
	let plugin = runtime.load(&args.dir)?;
	let info = plugin.info();
	println!("loaded {} v{}", info.id, info.version);

	let executed = plugin.run_declared_tests().await?;
	if executed.is_empty() {
		bail!("no Tests table declared");
	}

	for name in &executed {
		print!("{name:>24}: ");
		match plugin.run_single_test(name).await {
			Ok(()) => println!("ok"),
			Err(error) => println!("FAILED: {error}"),
		}
	}
	Ok(())
}

async fn smoke(args: &Args) -> anyhow::Result<()> {
	let source = load_plugin(args).await?;
	let info = source.info();
	println!(
		"{:>16}: {} v{} ({})",
		"info",
		info.name,
		info.version,
		match info.kind {
			source_sdk::WorkKindTag::Manga => "manga",
			source_sdk::WorkKindTag::Novel => "novel",
		}
	);

	let mut probe_url = args.work_url.clone();
	let mut chapter_probe: Option<String> = None;
	let mut cover_probe: Option<String> = None;
	let mut image_probe = Vec::new();

	probe(args, "latest", async {
		let hits = source.latest(1).await?;
		if probe_url.is_none() {
			if let Some(hit) = hits.first() {
				probe_url = Some(hit.remote_url.clone());
				cover_probe = hit.cover_url.clone();
			}
		}
		Ok(format!("{} works", hits.len()))
	})
	.await?;

	probe(args, "trending", async {
		let hits = source.trending(1).await?;
		Ok(format!("{} works", hits.len()))
	})
	.await?;

	probe(args, "search", async {
		let hits = source.search(&args.query, 1).await?;
		if probe_url.is_none() {
			if let Some(hit) = hits.first() {
				probe_url = Some(hit.remote_url.clone());
				cover_probe = hit.cover_url.clone();
			}
		}
		Ok(format!("{} hits for `{}`", hits.len(), args.query))
	})
	.await?;

	let Some(probe_url) = probe_url else {
		bail!("no work discovered; cannot probe details/chapters");
	};

	probe(args, "work", async {
		let details = source.fetch_work(&probe_url).await?;
		if details.cover_url.is_some() {
			cover_probe = details.cover_url.clone();
		}
		if let Some(last) = details.chapters.last() {
			chapter_probe = Some(last.remote_url.clone());
		}
		Ok(format!(
			"`{}` with {} chapters (first: {}; last: {}){}",
			details.title,
			details.chapters.len(),
			details.chapters.first().map(|chapter| chapter.title.as_str()).unwrap_or("-"),
			details.chapters.last().map(|chapter| chapter.title.as_str()).unwrap_or("-"),
			details
				.status
				.as_deref()
				.map(|status| format!(" ({status})"))
				.unwrap_or_default()
		))
	})
	.await?;
	let Some(chapter_url) = chapter_probe else {
		bail!("work exposed no chapters");
	};

	probe(args, "chapter", async {
		match source.fetch_chapter(&chapter_url).await? {
			domain::ChapterContent::Images(pages) => {
				image_probe = pages;
				Ok(format!("{} pages", image_probe.len()))
			}
			domain::ChapterContent::Html(html) => Ok(format!("{} bytes of html", html.len())),
		}
	})
	.await?;

	if let Some(dir) = &args.download_images {
		let referer = source
			.info()
			.referer_url
			.as_deref()
			.context("plugin does not declare referer_url")?;
		download_images(dir, referer, cover_probe.as_deref(), &image_probe).await?;
	}

	Ok(())
}

async fn download_images(dir: &Path, referer: &str, cover: Option<&str>, pages: &[String]) -> anyhow::Result<()> {
	let client = reqwest::Client::builder()
		.user_agent(IMAGE_USER_AGENT)
		.build()
		.context("building image client")?;
	tokio::fs::create_dir_all(dir).await?;
	let mut urls = Vec::with_capacity(1 + pages.len());
	if let Some(cover) = cover {
		urls.push(("cover".to_owned(), cover.to_owned()));
	}
	urls.extend(
		pages
			.iter()
			.enumerate()
			.map(|(index, url)| (format!("page-{index:04}"), url.clone())),
	);

	for (name, url) in urls {
		let parsed = reqwest::Url::parse(&url).with_context(|| format!("invalid image URL: {url}"))?;
		if !matches!(parsed.scheme(), "http" | "https") {
			bail!("unsupported image URL scheme: {url}");
		}
		let response = client
			.get(&url)
			.header(REFERER, HeaderValue::try_from(referer)?)
			.send()
			.await?
			.error_for_status()?;
		let extension = response
			.headers()
			.get("content-type")
			.and_then(|value| value.to_str().ok())
			.and_then(|value| value.strip_prefix("image/"))
			.and_then(|value| value.split(';').next())
			.unwrap_or_else(|| parsed.path().rsplit('.').next().unwrap_or("bin"));
		let path = dir.join(format!("{name}.{extension}"));
		tokio::fs::write(&path, response.bytes().await?).await?;
		println!("downloaded {} -> {}", url, path.display());
	}
	Ok(())
}

async fn probe(args: &Args, name: &'static str, body: impl Future<Output = anyhow::Result<String>>) -> anyhow::Result<()> {
	if args.only.as_deref().is_some_and(|only| !only.contains(name)) {
		return Ok(());
	}
	print!("{name:>16}: ");
	let start = Instant::now();
	match body.await {
		Ok(summary) => println!("PASS ({:.2?}) {summary}", start.elapsed()),
		Err(error) => println!("FAIL ({error})"),
	}
	Ok(())
}
