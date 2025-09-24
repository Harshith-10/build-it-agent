use anyhow::Result;
use clap::Parser;

mod executor;
mod language;
mod rusq;
mod types;

#[derive(Parser, Debug)]
#[command(name = "build-it-agent")]
#[command(about = "A multi-language code execution service")]
struct Args {
    /// Number of worker threads for job processing
    #[arg(short = 'w', long = "workers", help = "Number of worker threads (use 'max' for CPU count)")]
    workers: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Determine worker count
    let worker_count = match args.workers.as_deref() {
        Some("max") => num_cpus::get(),
        Some(count_str) => {
            count_str.parse::<usize>()
                .unwrap_or_else(|_| {
                    eprintln!("Invalid worker count '{}', using CPU count instead", count_str);
                    num_cpus::get()
                })
        },
        None => 4, // Default to 4 workers
    };

    println!("Starting build-it-agent with {} workers", worker_count);
    
    // Run executor with specified worker count
    executor::run(worker_count).await
}
