use anyhow::Result;
mod language_detector;

use language_detector::detect_languages;

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_path>", args[0]);
        return Ok(());
    }
    let config_path = &args[1];
    if !std::path::Path::new(config_path).exists() {
        eprintln!("Config file not found: {}", config_path);
        return Ok(());
    }
    let languages = detect_languages(&config_path)
        .await
        .into_iter()
        .map(|lang| lang.display_name)
        .collect::<Vec<String>>();

    println!("Detected languages: {:?}", languages);

    // monitor::run().await?;
    Ok(())
}
