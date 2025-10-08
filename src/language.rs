use std::collections::HashMap;
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
pub fn generate_language_configs() -> HashMap<String, LanguageConfig> {
    // Hardcoded language configurations (previously in `languages.json`).
    // Platform-specific differences are selected at runtime using cfg!(windows).
    let is_windows = cfg!(windows);
    let mut configs: HashMap<String, LanguageConfig> = HashMap::new();

    let ext_of = |fname: &str| -> String {
        Path::new(fname)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    };

    // python3
    {
        let file_name = "main.py".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "python3".to_string(),
            LanguageConfig {
                display_name: "Python 3".to_string(),
                file_name: file_name.clone(),
                version_command: "python3 --version".to_string(),
                compile_command: None,
                compile_args: vec![],
                run_command: if is_windows { "python" } else { "python3" }.to_string(),
                run_args: vec!["main.py".to_string()],
                file_extension: ext,
            },
        );
    }

    // python
    {
        let file_name = "main.py".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "python".to_string(),
            LanguageConfig {
                display_name: "Python".to_string(),
                file_name: file_name.clone(),
                version_command: "python --version".to_string(),
                compile_command: None,
                compile_args: vec![],
                run_command: "python".to_string(),
                run_args: vec!["main.py".to_string()],
                file_extension: ext,
            },
        );
    }

    // java
    {
        let file_name = "Main.java".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "java".to_string(),
            LanguageConfig {
                display_name: "Java".to_string(),
                file_name: file_name.clone(),
                version_command: "java -version".to_string(),
                compile_command: Some("javac".to_string()),
                compile_args: vec!["Main.java".to_string()],
                run_command: "java".to_string(),
                run_args: vec!["Main".to_string()],
                file_extension: ext,
            },
        );
    }

    // gcc
    {
        let file_name = "main.c".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "main.c".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
            ]
        } else {
            vec!["main.c".to_string(), "-o".to_string(), "main".to_string()]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "gcc".to_string(),
            LanguageConfig {
                display_name: "GNU C".to_string(),
                file_name: file_name.clone(),
                version_command: "gcc --version".to_string(),
                compile_command: Some("gcc".to_string()),
                compile_args,
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // clang
    {
        let file_name = "main.c".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "main.c".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
            ]
        } else {
            vec!["main.c".to_string(), "-o".to_string(), "main".to_string()]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "clang".to_string(),
            LanguageConfig {
                display_name: "Clang C".to_string(),
                file_name: file_name.clone(),
                version_command: "clang --version".to_string(),
                compile_command: Some("clang".to_string()),
                compile_args,
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // g++ (gpp)
    {
        let file_name = "main.cpp".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "main.cpp".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
            ]
        } else {
            vec!["main.cpp".to_string(), "-o".to_string(), "main".to_string()]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "gpp".to_string(),
            LanguageConfig {
                display_name: "GNU C++".to_string(),
                file_name: file_name.clone(),
                version_command: "g++ --version".to_string(),
                compile_command: Some("g++".to_string()),
                compile_args: compile_args.clone(),
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext.clone(),
            },
        );
    }

    // clang++
    {
        let file_name = "main.cpp".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "main.cpp".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
            ]
        } else {
            vec!["main.cpp".to_string(), "-o".to_string(), "main".to_string()]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "clangpp".to_string(),
            LanguageConfig {
                display_name: "Clang C++".to_string(),
                file_name: file_name.clone(),
                version_command: "clang++ --version".to_string(),
                compile_command: Some("clang++".to_string()),
                compile_args,
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // rust
    {
        let file_name = "main.rs".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "main.rs".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
            ]
        } else {
            vec!["main.rs".to_string(), "-o".to_string(), "main".to_string()]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "rust".to_string(),
            LanguageConfig {
                display_name: "Rust".to_string(),
                file_name: file_name.clone(),
                version_command: "rustc --version".to_string(),
                compile_command: Some("rustc".to_string()),
                compile_args,
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // javascript
    {
        let file_name = "main.js".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "javascript".to_string(),
            LanguageConfig {
                display_name: "JavaScript".to_string(),
                file_name: file_name.clone(),
                version_command: "node --version".to_string(),
                compile_command: None,
                compile_args: vec![],
                run_command: "node".to_string(),
                run_args: vec!["main.js".to_string()],
                file_extension: ext,
            },
        );
    }

    // go
    {
        let file_name = "main.go".to_string();
        let ext = ext_of(&file_name);
        let compile_args = if is_windows {
            vec![
                "build".to_string(),
                "-o".to_string(),
                "main.exe".to_string(),
                "main.go".to_string(),
            ]
        } else {
            vec![
                "build".to_string(),
                "-o".to_string(),
                "main".to_string(),
                "main.go".to_string(),
            ]
        };
        let run_command = if is_windows { "main.exe" } else { "./main" };
        configs.insert(
            "go".to_string(),
            LanguageConfig {
                display_name: "Go".to_string(),
                file_name: file_name.clone(),
                version_command: "go version".to_string(),
                compile_command: Some("go".to_string()),
                compile_args,
                run_command: run_command.to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // csharp
    {
        let file_name = "Program.cs".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "csharp".to_string(),
            LanguageConfig {
                display_name: "C# (.NET)".to_string(),
                file_name: file_name.clone(),
                version_command: "dotnet --version".to_string(),
                compile_command: Some("dotnet".to_string()),
                compile_args: vec!["build".to_string()],
                run_command: "dotnet".to_string(),
                run_args: vec!["run".to_string()],
                file_extension: ext,
            },
        );
    }

    // psql
    {
        let file_name = "".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "psql".to_string(),
            LanguageConfig {
                display_name: "PostgreSQL (psql)".to_string(),
                file_name: file_name.clone(),
                version_command: "psql --version".to_string(),
                compile_command: None,
                compile_args: vec![],
                run_command: "psql".to_string(),
                run_args: vec![],
                file_extension: ext,
            },
        );
    }

    // kotlin
    {
        let file_name = "Main.kt".to_string();
        let ext = ext_of(&file_name);
        configs.insert(
            "kotlin".to_string(),
            LanguageConfig {
                display_name: "Kotlin".to_string(),
                file_name: file_name.clone(),
                version_command: "kotlinc -version".to_string(),
                compile_command: Some("kotlinc".to_string()),
                compile_args: vec![
                    "Main.kt".to_string(),
                    "-include-runtime".to_string(),
                    "-d".to_string(),
                    "Main.jar".to_string(),
                ],
                run_command: "java".to_string(),
                run_args: vec!["-jar".to_string(), "Main.jar".to_string()],
                file_extension: ext,
            },
        );
    }

    configs
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_language_configs_not_empty() {
        let configs = generate_language_configs();
        assert!(!configs.is_empty(), "Language configs should not be empty");
    }

    #[test]
    fn test_common_languages_present() {
        let configs = generate_language_configs();
        
        // Test that common languages are configured
        assert!(configs.contains_key("python3"), "Python3 should be configured");
        assert!(configs.contains_key("java"), "Java should be configured");
        assert!(configs.contains_key("gcc"), "GCC should be configured");
        assert!(configs.contains_key("gpp"), "G++ should be configured");
    }

    #[test]
    fn test_python3_config() {
        let configs = generate_language_configs();
        let python_config = configs.get("python3").expect("Python3 config should exist");

        assert_eq!(python_config.display_name, "Python 3");
        assert_eq!(python_config.file_name, "main.py");
        assert_eq!(python_config.file_extension, "py");
        assert!(python_config.compile_command.is_none());
    }

    #[test]
    fn test_java_config() {
        let configs = generate_language_configs();
        let java_config = configs.get("java").expect("Java config should exist");

        assert_eq!(java_config.display_name, "Java");
        assert_eq!(java_config.file_name, "Main.java");
        assert_eq!(java_config.file_extension, "java");
        assert_eq!(java_config.compile_command, Some("javac".to_string()));
    }

    #[test]
    fn test_c_config() {
        let configs = generate_language_configs();
        let c_config = configs.get("gcc").expect("GCC config should exist");

        assert_eq!(c_config.display_name, "GNU C");
        assert_eq!(c_config.file_name, "main.c");
        assert_eq!(c_config.file_extension, "c");
        assert_eq!(c_config.compile_command, Some("gcc".to_string()));
    }

    #[test]
    fn test_cpp_config() {
        let configs = generate_language_configs();
        let cpp_config = configs.get("gpp").expect("G++ config should exist");

        assert_eq!(cpp_config.display_name, "GNU C++");
        assert_eq!(cpp_config.file_name, "main.cpp");
        assert_eq!(cpp_config.file_extension, "cpp");
        assert_eq!(cpp_config.compile_command, Some("g++".to_string()));
    }

    #[test]
    fn test_language_config_file_extensions() {
        let configs = generate_language_configs();
        
        // Verify file extensions are correctly extracted
        for (_name, config) in configs.iter() {
            if !config.file_name.is_empty() {
                // If there's a file extension, it should match
                let expected_ext = std::path::Path::new(&config.file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                assert_eq!(config.file_extension, expected_ext);
            }
        }
    }

    #[test]
    fn test_compiled_languages_have_compile_command() {
        let configs = generate_language_configs();
        
        // Compiled languages should have compile commands
        let compiled_langs = vec!["java", "gcc", "gpp", "clang", "clangpp"];
        for lang in compiled_langs {
            if let Some(config) = configs.get(lang) {
                assert!(
                    config.compile_command.is_some(),
                    "{} should have a compile command",
                    lang
                );
            }
        }
    }

    #[test]
    fn test_interpreted_languages_no_compile_command() {
        let configs = generate_language_configs();
        
        // Interpreted languages should not have compile commands
        let interpreted_langs = vec!["python3", "python"];
        for lang in interpreted_langs {
            if let Some(config) = configs.get(lang) {
                assert!(
                    config.compile_command.is_none(),
                    "{} should not have a compile command",
                    lang
                );
            }
        }
    }

    #[test]
    fn test_language_info_creation() {
        let info = LanguageInfo {
            name: "python3".to_string(),
            display_name: "Python 3".to_string(),
            version: "Python 3.9.0".to_string(),
        };

        assert_eq!(info.name, "python3");
        assert_eq!(info.display_name, "Python 3");
        assert_eq!(info.version, "Python 3.9.0");
    }

    #[test]
    fn test_platform_specific_python_command() {
        let configs = generate_language_configs();
        let python_config = configs.get("python3").expect("Python3 config should exist");

        if cfg!(windows) {
            assert_eq!(python_config.run_command, "python");
        } else {
            assert_eq!(python_config.run_command, "python3");
        }
    }

    #[test]
    fn test_all_configs_have_display_name() {
        let configs = generate_language_configs();
        
        for (name, config) in configs.iter() {
            assert!(
                !config.display_name.is_empty(),
                "Language {} should have a display name",
                name
            );
        }
    }

    #[test]
    fn test_all_configs_have_version_command() {
        let configs = generate_language_configs();
        
        for (name, config) in configs.iter() {
            if name != "psql" { // psql might have different requirements
                assert!(
                    !config.version_command.is_empty(),
                    "Language {} should have a version command",
                    name
                );
            }
        }
    }
}

