use crate::language::LanguageConfig;
use crate::types::{CaseResult, ExecuteRequest, ExecuteResponse};
use anyhow::Result;
use std::time::Instant;
use tempfile;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time;

pub async fn execute(
    request: ExecuteRequest,
    lang_config: LanguageConfig,
) -> Result<ExecuteResponse> {
    let temp_dir = tempfile::tempdir()?;
    let work_dir = temp_dir.path().to_path_buf();

    let file_name = format!("main.{}", lang_config.file_extension);
    let source_path = work_dir.join(&file_name);
    tokio::fs::write(&source_path, &request.code).await?;

    let mut compiled = false;
    let mut details = None;

    if let Some(compile_command) = &lang_config.compile_command {
        let mut cmd = Command::new(compile_command);
        cmd.current_dir(&work_dir);
        cmd.args(&lang_config.compile_args);
        let output = cmd.output().await?;

        if !output.status.success() {
            details = Some(String::from_utf8_lossy(&output.stderr).to_string());
            return Ok(ExecuteResponse {
                compiled: false,
                language: request.language,
                details,
                results: vec![],
                total_duration_ms: 0,
            });
        }
        compiled = true;
    }

    let mut results = vec![];
    let mut total_duration_ms = 0;

    for testcase in request.testcases {
        let mut cmd = Command::new(&lang_config.run_command);
        cmd.current_dir(&work_dir);
        cmd.args(&lang_config.run_args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;

        let start_time = Instant::now();

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(testcase.input.as_bytes()).await?;
        }

        let timeout = time::Duration::from_millis(testcase.timeout_ms.unwrap_or(5000));
        // Wait with timeout; on timeout, kill child
        let mut timed_out = false;
        let status = if let Ok(res) = time::timeout(timeout, child.wait()).await {
            match res {
                Ok(s) => s,
                Err(e) => return Err(e.into()),
            }
        } else {
            child.kill().await?;
            timed_out = true;
            child.wait().await?
        };
        // We do not capture stdout/stderr for simplicity
        let stdout = String::new();
        let stderr = String::new();

        let duration_ms = start_time.elapsed().as_millis() as u64;
        total_duration_ms += duration_ms;

        let passed = if let Some(expected) = &testcase.expected {
            stdout == *expected
        } else {
            false
        };
        // Determine ok and exit_code from output_opt
        let ok = status.success() && !timed_out;
        let exit_code = status.code();
        results.push(CaseResult {
            id: testcase.id,
            ok,
            passed,
            stdout,
            stderr,
            timed_out,
            duration_ms,
            memory_kb: 0, // Simplified: not implemented
            exit_code,
            term_signal: None, // Simplified: not implemented
        });
    }

    Ok(ExecuteResponse {
        compiled,
        language: request.language,
        details,
        results,
        total_duration_ms,
    })
}
