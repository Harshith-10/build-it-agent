use anyhow::Result;
use tokio::sync::oneshot;
mod executor;
mod language;
mod monitor;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    let (executor_ready_tx, executor_ready_rx) = oneshot::channel();
    let (monitor_ready_tx, monitor_ready_rx) = oneshot::channel();

    // Run both services in parallel
    tokio::try_join!(
        async move {
            executor::run(Some(executor_ready_tx)).await
        },
        async move {
            monitor::run(Some(monitor_ready_tx)).await
        },
        async move {
            // Wait for both services to report readiness before printing the banner.
            executor_ready_rx.await?;
            monitor_ready_rx.await?;
            println!(
                "\n🟢 BuildIT Agent is running...\n⚠️ WARNING: Do NOT close this window until your exam is completed, else it will be terminated!"
            );
            Ok(())
        }
    )?;
    Ok(())
}
