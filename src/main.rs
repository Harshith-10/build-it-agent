use anyhow::Result;
mod language;
mod monitor;
mod executor;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    // Run both services in parallel
    tokio::try_join!(
        async { executor::run().await },
        async { monitor::run().await },
    )?;
    Ok(())
}
