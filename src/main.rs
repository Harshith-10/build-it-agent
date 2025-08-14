use anyhow::Result;
mod monitor;
mod queue;

#[tokio::main]
async fn main() -> Result<()> {
    monitor::run().await?;
    Ok(())
}
