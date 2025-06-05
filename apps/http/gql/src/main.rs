#[tokio::main]
async fn main() -> anyhow::Result<()> {
	gql_api::run().await
}
