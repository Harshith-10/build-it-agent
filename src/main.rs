use anyhow::Result;
mod language_detector;
mod local_code_exec;

use language_detector::detect_languages;
use local_code_exec::CodeExecutor;

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        eprintln!("Usage: {} <config_path> [language] [source_file]", args[0]);
        return Ok(());
    }
    let config_path = &args[1];
    if !std::path::Path::new(config_path).exists() {
        eprintln!("Config file not found: {}", config_path);
        return Ok(());
    }

    // Detect available languages
    let languages = detect_languages(&config_path).await;
    let available_languages: Vec<String> = languages
        .iter()
        .filter(|lang| lang.found)
        .map(|lang| format!("{} ({})", lang.display_name, lang.codename))
        .collect();

    println!("Available languages: {:?}", available_languages);

    // If language and source file are provided, execute code
    if args.len() >= 4 {
        let language = &args[2];
        let source_file = &args[3];
        
        println!("\nAttempting to execute {} code from file: {}", language, source_file);
        
        match CodeExecutor::new(config_path) {
            Ok(executor) => {
                match executor.execute(language, source_file, None) {
                    Ok(output) => {
                        println!("Execution successful!");
                        println!("Output:\n{}", output);
                    }
                    Err(e) => {
                        eprintln!("Execution failed: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create code executor: {}", e);
            }
        }
    }

    // monitor::run().await?;
    Ok(())
}
