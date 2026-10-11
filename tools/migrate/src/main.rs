use manga_vault_migrate::legacy;

fn usage() -> String {
	eprintln!(
		"usage: manga-vault-migrate --from <mysql://user:pass@host:3306/db | sqlite:///path/vault.db> --to <DATABASE_URL>"
	);
	eprintln!("       the target accepts sqlite:// or postgres://");
	std::process::exit(2)
}

fn arg_value(args: &[String], flag: &str) -> String {
	args.iter()
		.position(|arg| arg == flag)
		.and_then(|index| args.get(index + 1))
		.cloned()
		.unwrap_or_else(usage)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::fmt().with_env_filter("info,sqlx=warn").init();
	let args: Vec<String> = std::env::args().collect();
	let from = arg_value(&args, "--from");
	let to = arg_value(&args, "--to");

	let source = legacy::LegacyVault::connect(&from).await?;
	tracing::info!("legacy source is {}", source.name());
	let report = manga_vault_migrate::migrate(&source, &to).await?;

	println!("{:<22} {:>10} {:>10}", "entity", "legacy", "new");
	for entity in &report.entities {
		let status = if entity.matches() { "ok" } else { "MISMATCH" };
		println!("{:<22} {:>10} {:>10}  {status}", entity.label, entity.legacy, entity.migrated);
	}
	println!(
		"{} duplicate chapter rows collapsed onto their first occurrence",
		report.duplicate_chapters
	);
	println!(
		"{} re-read progress rows collapsed onto their first occurrence",
		report.duplicate_reads
	);

	if !report.verified() {
		eprintln!("verification failed: keep the legacy database as cold backup and investigate before retiring it");
		std::process::exit(1);
	}
	println!("migration verified; keep the legacy database as a cold backup");
	Ok(())
}
