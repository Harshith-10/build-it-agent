use anyhow::Result;
mod language;
mod monitor;

use language::{
    LanguageInfo,
    get_installed_languages,
    load_language_configs
};

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
    let configs = load_language_configs(config_path).unwrap();
    let languages = get_installed_languages(&configs)
        .await
        .into_iter()
        .map(|lang: LanguageInfo| lang.display_name)
        .collect::<Vec<String>>();

    println!("Detected languages: {:?}", languages);

    monitor::run().await?;
    Ok(())
}
