use clap::Parser;

#[derive(Parser)]
struct Args {
	url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();
	persistence::connect(&args.url).await?;
	println!("schema ready on {}", args.url);
	Ok(())
}
