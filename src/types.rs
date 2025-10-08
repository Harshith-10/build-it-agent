use serde::{Deserialize, Serialize};

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
    pub input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
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
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Success,
    Error,
    Timeout,
    CompileError,
    RuntimeError,
    UnsupportedLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub compiled: bool,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ExecutionStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub results: Vec<CaseResult>,
    pub total_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testcase_serialization() {
        let test_case = TestCase {
            id: 1,
            input: "hello".to_string(),
            expected: Some("world".to_string()),
            timeout_ms: Some(5000),
        };

        let json = serde_json::to_string(&test_case).unwrap();
        let deserialized: TestCase = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, 1);
        assert_eq!(deserialized.input, "hello");
        assert_eq!(deserialized.expected, Some("world".to_string()));
        assert_eq!(deserialized.timeout_ms, Some(5000));
    }

    #[test]
    fn test_testcase_default_timeout() {
        let json = r#"{"id":1,"input":"test","expected":null}"#;
        let test_case: TestCase = serde_json::from_str(json).unwrap();
        assert_eq!(test_case.timeout_ms, None);
    }

    #[test]
    fn test_execute_request_serialization() {
        let request = ExecuteRequest {
            language: "python3".to_string(),
            code: "print('hello')".to_string(),
            testcases: vec![
                TestCase {
                    id: 1,
                    input: "".to_string(),
                    expected: Some("hello".to_string()),
                    timeout_ms: None,
                }
            ],
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExecuteRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.language, "python3");
        assert_eq!(deserialized.code, "print('hello')");
        assert_eq!(deserialized.testcases.len(), 1);
    }

    #[test]
    fn test_case_result_creation() {
        let result = CaseResult {
            id: 1,
            ok: true,
            passed: true,
            input: "test input".to_string(),
            expected: Some("expected output".to_string()),
            stdout: "actual output".to_string(),
            stderr: "".to_string(),
            timed_out: false,
            duration_ms: 100,
            memory_kb: 1024,
            exit_code: Some(0),
            term_signal: None,
        };

        assert_eq!(result.id, 1);
        assert!(result.ok);
        assert!(result.passed);
        assert!(!result.timed_out);
        assert_eq!(result.duration_ms, 100);
    }

    #[test]
    fn test_execution_status_serialization() {
        let statuses = vec![
            ExecutionStatus::Success,
            ExecutionStatus::Error,
            ExecutionStatus::Timeout,
            ExecutionStatus::CompileError,
            ExecutionStatus::RuntimeError,
            ExecutionStatus::UnsupportedLanguage,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let _deserialized: ExecutionStatus = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_execute_response_with_results() {
        let response = ExecuteResponse {
            compiled: true,
            language: "python3".to_string(),
            status: Some(ExecutionStatus::Success),
            message: None,
            results: vec![
                CaseResult {
                    id: 1,
                    ok: true,
                    passed: true,
                    input: "".to_string(),
                    expected: None,
                    stdout: "output".to_string(),
                    stderr: "".to_string(),
                    timed_out: false,
                    duration_ms: 50,
                    memory_kb: 512,
                    exit_code: Some(0),
                    term_signal: None,
                }
            ],
            total_duration_ms: 50,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ExecuteResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.compiled);
        assert_eq!(deserialized.results.len(), 1);
        assert_eq!(deserialized.total_duration_ms, 50);
    }

    #[test]
    fn test_execute_response_empty_results() {
        let response = ExecuteResponse {
            compiled: false,
            language: "unknown".to_string(),
            status: Some(ExecutionStatus::UnsupportedLanguage),
            message: Some("Language not supported".to_string()),
            results: vec![],
            total_duration_ms: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(!json.contains("\"results\""));
    }
}
