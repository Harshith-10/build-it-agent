use anyhow::Result;
mod language;
mod monitor;

use language::{
    LanguageInfo,
    get_installed_languages,
    generate_language_configs
};

#[tokio::main]
async fn main() -> Result<()> {
    let configs = generate_language_configs();
    let languages = get_installed_languages(&configs)
        .await
        .into_iter()
        .map(|lang: LanguageInfo| lang.display_name)
        .collect::<Vec<String>>();

    println!("Detected languages: {:?}", languages);

    monitor::run().await?;
    Ok(())
}
