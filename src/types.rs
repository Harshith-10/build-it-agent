use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Rust,
    GNU_C,
    GNU_Cpp,
    Clang_C,
    Clang_Cpp,
    Python,
    Python3,
    Java,
    Kotlin,
    JavaScript,
    Go,
    CSharp,
    Postgres
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::GNU_C => "gcc",
            Language::GNU_Cpp => "gpp",
            Language::Clang_C => "clang",
            Language::Clang_Cpp => "clangpp",
            Language::Python => "python",
            Language::Python3 => "python3",
            Language::Java => "java",
            Language::Kotlin => "kotlin",
            Language::JavaScript => "javascript",
            Language::Go => "go",
            Language::CSharp => "csharp",
            Language::Postgres => "postgres"
        }
    }

    pub fn from_str(s: &str) -> Option<Language> {
        match s {
            "rust" => Some(Language::Rust),
            "gcc" => Some(Language::GNU_C),
            "gpp" => Some(Language::GNU_Cpp),
            "clang" => Some(Language::Clang_C),
            "clangpp" => Some(Language::Clang_Cpp),
            "python" => Some(Language::Python),
            "python3" => Some(Language::Python3),
            "java" => Some(Language::Java),
            "kotlin" => Some(Language::Kotlin),
            "javascript" => Some(Language::JavaScript),
            "go" => Some(Language::Go),
            "csharp" => Some(Language::CSharp),
            "postgres" => Some(Language::Postgres),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub id: i32,
    pub input: String,
    pub expected: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub language: String,
    pub code: String,
    pub testcases: Vec<TestCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub id: i32,
    pub ok: bool,
    pub passed: bool,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub duration_ms: u64,
    pub memory_kb: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term_signal: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub compiled: bool,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub results: Vec<CaseResult>,
    pub total_duration_ms: u64,
}