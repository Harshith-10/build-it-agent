use anyhow::Result;
mod executor;
mod language;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    // Run both services in parallel
    tokio::try_join!(
        async {
            executor::run().await
        },
    )?;
    Ok(())
}
