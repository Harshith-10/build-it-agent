use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Configuration used at runtime for each language
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LanguageConfig {
    pub display_name: String,
    pub file_name: String,
    pub version_command: String,
    pub compile_command: Option<String>,
    pub compile_args: Vec<String>,
    pub run_command: String,
    pub run_args: Vec<String>,
    pub file_extension: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct LanguageInfo {
    pub name: String,
    pub display_name: String,
    pub version: String,
}

// Load language configurations from JSON and select platform-specific settings
pub fn load_language_configs(config_path: &str) -> Result<HashMap<String, LanguageConfig>> {
    let data = fs::read_to_string(config_path)?;
    let v: Value = serde_json::from_str(&data)?;
    let obj = v
        .as_object()
        .ok_or_else(|| anyhow!("Invalid JSON structure"))?;
    let mut configs = HashMap::new();
    let os_key = if cfg!(windows) { "windows" } else { "unix" };
    for (name, entry) in obj {
        let entry_obj = if let Some(map) = entry.as_object() {
            map
        } else {
            continue;
        };
        let display_name = entry_obj
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let file_name = entry_obj
            .get("file_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let ext = Path::new(&file_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if let Some(platforms) = entry_obj.get("platforms").and_then(|v| v.as_object()) {
            if let Some(platform) = platforms.get(os_key).and_then(|v| v.as_object()) {
                if let Some(run_cmd) = platform.get("run_command").and_then(|v| v.as_str()) {
                    let run_args = platform
                        .get("run_args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let compile_command = platform
                        .get("compile_command")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let compile_args = platform
                        .get("compile_args")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let version_command = entry_obj
                        .get("version_command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    configs.insert(
                        name.clone(),
                        LanguageConfig {
                            display_name: display_name.clone(),
                            file_name: file_name.clone(),
                            version_command,
                            compile_command,
                            compile_args,
                            run_command: run_cmd.to_string(),
                            run_args,
                            file_extension: ext,
                        },
                    );
                }
            }
        }
    }
    Ok(configs)
}

// Get supported language info (cross-platform)
// Runs each language's configured `version_command` via the platform shell so commands
// containing flags or complex expressions work (e.g. "python --version").
pub async fn get_installed_languages(
    configs: &HashMap<String, LanguageConfig>,
) -> Vec<LanguageInfo> {
    use futures::stream::{FuturesUnordered, StreamExt};
    use tokio::process::Command as TokioCommand;
    use tokio::time::{timeout, Duration};

    let mut tasks = FuturesUnordered::new();

    for (name, cfg) in configs.iter() {
        let name = name.clone();
        let display = cfg.display_name.clone();
        let cmd_str = cfg.version_command.trim().to_string();
        if cmd_str.is_empty() {
            continue;
        }

        // Spawn an async task per language detection command.
        tasks.push(async move {
            // Use the platform shell so complex commands / flags work.
            let mut cmd = if cfg!(windows) {
                let mut c = TokioCommand::new("cmd");
                c.args(&["/C", &cmd_str]);
                c
            } else {
                let mut c = TokioCommand::new("sh");
                c.arg("-c").arg(&cmd_str);
                c
            };

            // Give each check a short timeout so a hanging tool won't block discovery.
            let run = async {
                match cmd.output().await {
                    Ok(out) => {
                        let combined = format!(
                            "{}{}",
                            String::from_utf8_lossy(&out.stdout),
                            String::from_utf8_lossy(&out.stderr)
                        );

                        if combined.contains("not found") || combined.contains("not recognized") {
                            return None;
                        }

                        if let Some(ver_line) = combined.lines().find(|l| !l.trim().is_empty()) {
                            let version = ver_line.trim().to_string();
                            return Some(LanguageInfo {
                                name: name.clone(),
                                display_name: display.clone(),
                                version,
                            });
                        }
                        None
                    }
                    Err(_) => None,
                }
            };

            // 3 second timeout per language detection (reasonable default)
            match timeout(Duration::from_secs(3), run).await {
                Ok(opt) => opt,
                Err(_) => None,
            }
        });
    }

    let mut result = Vec::new();
    while let Some(opt) = tasks.next().await {
        if let Some(lang) = opt {
            result.push(lang);
        }
    }

    result
}
