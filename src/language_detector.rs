use std::process::Command;
use tokio::task;
use futures::future::join_all;
use which::which;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use serde_json;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug)]
pub struct LanguagePresence {
    pub codename: &'static str,
    pub display_name: String,
    pub found: bool,
    pub version: Option<String>,
    pub path: Option<String>,
}

// NEW: Struct for platform-specific command configurations.
#[derive(Debug, Deserialize, Clone)]
struct PlatformConfig {
    run_command: String,
    #[allow(dead_code)]
    run_args: Vec<String>,
    compile_command: Option<String>,
    #[allow(dead_code)]
    compile_args: Vec<String>,
}

// NEW: Struct to hold configurations for different platforms from JSON.
#[derive(Debug, Deserialize, Clone)]
struct Platforms {
    windows: PlatformConfig,
    unix: PlatformConfig,
}

// MODIFIED: LanguageConfig to correctly parse the nested platform data from the JSON file.
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct LanguageConfig {
    display_name: String,
    #[allow(dead_code)]
    file_name: String,
    version_commands: Vec<String>,
    platforms: Platforms, // This now correctly maps to the "platforms" object in the JSON.
    #[allow(dead_code)]
    optimization_args: Vec<String>,
}

fn find_executable(exe: &str) -> std::io::Result<PathBuf> {
    which(exe).map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))
}

/// Detect languages based on commands listed in a JSON config file, in parallel using Tokio.
#[allow(dead_code)]
pub async fn detect_languages(config_path: &str) -> Vec<LanguagePresence> {
    // Load and parse the JSON configuration
    let mut file = match File::open(config_path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };
    let mut content = String::new();
    if file.read_to_string(&mut content).is_err() {
        return Vec::new();
    }
    // Deserialize JSON to the new, corrected LanguageConfig entries.
    let mapping: HashMap<String, LanguageConfig> = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return Vec::new(), // This was being triggered due to the struct mismatch.
    };

    let mut tasks = Vec::new();
    for (codename, cfg) in mapping {
        let codename_clone = codename.clone();
        let cfg_clone = cfg.clone();
        tasks.push(task::spawn_blocking(move || {
            // MODIFIED: Select the correct platform configuration based on the target OS.
            let platform_cfg = if cfg!(target_os = "windows") {
                cfg_clone.platforms.windows
            } else {
                cfg_clone.platforms.unix
            };

            let name_static: &'static str = Box::leak(codename_clone.into_boxed_str());
            let mut found = false;
            let mut version = None;
            let mut path = None;

            // MODIFIED: Determine the primary executable from the platform-specific config.
            if let Some(primary) = platform_cfg.compile_command.as_ref().or(Some(&platform_cfg.run_command)) {
                if let Ok(exec_path) = find_executable(primary) {
                    found = true;
                    path = Some(exec_path.to_string_lossy().to_string());
                    // Retrieve version via version_commands
                    for cmd in &cfg_clone.version_commands {
                        let mut parts = cmd.split_whitespace();
                        if let Some(exe) = parts.next() {
                            let args: Vec<&str> = parts.collect();
                            if let Ok(ver_path) = find_executable(exe) {
                                if let Ok(output) = Command::new(&ver_path).args(&args).output() {
                                    let out_bytes = if !output.stdout.is_empty() { output.stdout } else { output.stderr };
                                    version = Some(String::from_utf8_lossy(&out_bytes).trim().to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            LanguagePresence {
                codename: name_static,
                display_name: cfg_clone.display_name.clone(),
                found,
                version,
                path,
            }
        }));
    }
    let results = join_all(tasks).await.into_iter().filter_map(|r| r.ok()).collect();
    results
}