use std::process::Command;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use serde::Deserialize;
use anyhow::{Result, anyhow};

// Platform-specific configuration
#[derive(Debug, Deserialize, Clone)]
struct PlatformConfig {
    compile_command: Option<String>,
    compile_args: Vec<String>,
    run_command: String,
    run_args: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct LanguageConfig {
    display_name: String,
    file_name: String,
    version_commands: Vec<String>,
    platforms: HashMap<String, PlatformConfig>,
    optimization_args: Vec<String>,
}

pub struct CodeExecutor {
    config: HashMap<String, LanguageConfig>,
}

impl CodeExecutor {
    pub fn new(config_path: &str) -> Result<Self> {
        let mut file = File::open(config_path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        let config: HashMap<String, LanguageConfig> = serde_json::from_str(&content)?;
        
        Ok(CodeExecutor { config })
    }

    pub fn execute(&self, language: &str, source_file: &str, additional_args: Option<Vec<String>>) -> Result<String> {
        let lang_config = self.config.get(language)
            .ok_or_else(|| anyhow!("Language '{}' not found in configuration", language))?;
        
        // Determine current platform
        let platform = if cfg!(windows) { "windows" } else { "unix" };
        
        let platform_config = lang_config.platforms.get(platform)
            .ok_or_else(|| anyhow!("Platform '{}' not supported for language '{}'", platform, language))?;

        // Compile if necessary
        if let Some(compile_cmd) = &platform_config.compile_command {
            let mut compile_args = platform_config.compile_args.clone();
            if let Some(ref extra_args) = additional_args {
                compile_args.extend(extra_args.clone());
            }
            
            let compile_output = Command::new(compile_cmd)
                .args(&compile_args)
                .output()?;
            
            if !compile_output.status.success() {
                return Err(anyhow!("Compilation failed: {}", String::from_utf8_lossy(&compile_output.stderr)));
            }
        }

        // Execute the program
        let mut run_args = platform_config.run_args.clone();
        
        // For MySQL, we need special handling to execute SQL files
        if language == "mysql" {
            // Add source file execution for MySQL
            run_args.extend(vec!["-e".to_string(), format!("source {}", source_file)]);
        } else if !source_file.is_empty() && !run_args.contains(&source_file.to_string()) {
            // For other languages, if source file isn't already in args, add it
            if platform_config.run_command != source_file {
                run_args.push(source_file.to_string());
            }
        }
        
        if let Some(ref extra_args) = additional_args {
            run_args.extend(extra_args.clone());
        }

        let output = Command::new(&platform_config.run_command)
            .args(&run_args)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow!("Execution failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        self.config.keys().cloned().collect()
    }

    pub fn get_file_extension(&self, language: &str) -> Option<String> {
        self.config.get(language).map(|cfg| {
            let file_name = &cfg.file_name;
            if let Some(dot_pos) = file_name.rfind('.') {
                file_name[dot_pos..].to_string()
            } else {
                String::new()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_code_executor_creation() {
        // This test assumes the languages.json file exists
        if std::path::Path::new("languages.json").exists() {
            let executor = CodeExecutor::new("languages.json");
            assert!(executor.is_ok());
        }
    }

    #[test]
    fn test_get_supported_languages() {
        if std::path::Path::new("languages.json").exists() {
            let executor = CodeExecutor::new("languages.json").unwrap();
            let languages = executor.get_supported_languages();
            assert!(languages.contains(&"mysql".to_string()));
        }
    }
}
