#[tokio::main]
async fn main() -> anyhow::Result<()> {
    spell_check::cli::run().await
}
