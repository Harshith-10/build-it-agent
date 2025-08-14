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

// Detailed config for each language from JSON
// Unused fields are for future execution logic
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
struct LanguageConfig {
    display_name: String,
    file_name: String,
    version_commands: Vec<String>,
    compile_command: Option<String>,
    compile_args: Vec<String>,
    optimization_args: Vec<String>,
    run_command: String,
    run_args: Vec<String>,
}

fn find_executable(exe: &str) -> std::io::Result<PathBuf> {
    which(exe).map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e))
}

/// Detect languages based on commands listed in a JSON config file, in parallel using Tokio
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
    // Deserialize JSON to detailed LanguageConfig entries
    let mapping: HashMap<String, LanguageConfig> = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    let mut tasks = Vec::new();
    for (codename, cfg) in mapping {
        let codename_clone = codename.clone();
        let cfg_clone = cfg.clone();
        tasks.push(task::spawn_blocking(move || {
            let name_static: &'static str = Box::leak(codename_clone.into_boxed_str());
            let mut found = false;
            let mut version = None;
            let mut path = None;
            // Determine primary executable: compile or run
            if let Some(primary) = cfg_clone.compile_command.as_ref().or(Some(&cfg_clone.run_command)) {
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
