use anyhow::Result;
mod executor;
mod language;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = executor::run().await;
    Ok(())
}
