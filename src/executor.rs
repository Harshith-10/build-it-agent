use crate::language::{generate_language_configs, get_installed_languages, LanguageConfig};
use crate::types::{CaseResult, ExecuteRequest, ExecuteResponse, ExecutionStatus};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use tower_http::cors;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;
use tempfile;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::{mpsc, RwLock};
use tokio::time;

#[derive(Clone)]
struct AppState {
    configs: Arc<HashMap<String, LanguageConfig>>, // language key -> config
    available: Arc<HashSet<String>>,               // installed language keys
    langs_list: Arc<Vec<LanguageSummary>>,         // for GET /languages
    jobs: Arc<RwLock<HashMap<u64, JobState>>>,
    sender: mpsc::Sender<(u64, ExecuteRequest)>,
    next_id: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct IdResponse {
    id: u64,
}

#[derive(Debug, Clone, Serialize)]
struct LanguageSummary {
    display_name: String,
    language: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
enum JobStatusResponse {
    Queued,
    Running,
    Completed { result: ExecuteResponse },
    Error { error: String },
}

#[derive(Debug, Clone)]
enum JobState {
    Queued,
    Running,
    Completed(ExecuteResponse),
    Error(String),
}

pub async fn run() -> Result<()> {
    // Build language configs and detect installed ones once at startup
    let configs = generate_language_configs();
    let installed = get_installed_languages(&configs).await;
    let available: HashSet<String> = installed.iter().map(|li| li.name.clone()).collect();
    let langs_list: Vec<LanguageSummary> = installed
        .into_iter()
        .map(|li| LanguageSummary {
            display_name: li.display_name,
            language: li.name,
        })
        .collect();

    println!(
        "Executor detected languages: {:?}",
        langs_list
            .iter()
            .map(|l| format!("{} ({})", l.display_name, l.language))
            .collect::<Vec<_>>()
    );

    let (tx, rx) = mpsc::channel::<(u64, ExecuteRequest)>(100);
    let state = AppState {
        configs: Arc::new(configs),
        available: Arc::new(available),
        langs_list: Arc::new(langs_list),
        jobs: Arc::new(RwLock::new(HashMap::new())),
        sender: tx,
        next_id: Arc::new(AtomicU64::new(1)),
    };

    // Spawn worker loop
    tokio::spawn(worker_loop(state.clone(), rx));

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/languages", get(languages_handler))
        .route("/execute", post(enqueue_handler))
        .route("/status/:id", get(status_handler))
        .with_state(state)
        .layer(
            cors::CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_methods(cors::Any)
                .allow_headers(cors::Any),
        );

    let port = 8910;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("Executor listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn worker_loop(state: AppState, mut rx: mpsc::Receiver<(u64, ExecuteRequest)>) {
    while let Some((id, req)) = rx.recv().await {
        {
            let mut jobs = state.jobs.write().await;
            jobs.insert(id, JobState::Running);
        }

        let res = execute_request(&req, &state).await;
        let mut jobs = state.jobs.write().await;
        match res {
            Ok(resp) => {
                jobs.insert(id, JobState::Completed(resp));
            }
            Err(e) => {
                jobs.insert(id, JobState::Error(e.to_string()));
            }
        }
    }
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(HealthResponse { status: "ok" }))
}

async fn languages_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Clone the inner Vec to avoid lifetime issues and Arc serialization concerns
    let list: Vec<LanguageSummary> = state.langs_list.as_ref().clone();
    Json(list)
}

async fn enqueue_handler(
    State(state): State<AppState>,
    Json(req): Json<ExecuteRequest>,
) -> Response {
    // Validate requested language is available
    if !state.available.contains(&req.language) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Unsupported or unavailable language: {}", req.language)
            })),
        )
            .into_response();
    }

    // Normalize language casing to exact key
    // (no-op because we expect correct key)
    let id = state.next_id.fetch_add(1, Ordering::Relaxed);
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(id, JobState::Queued);
    }
    // Ensure code is written against the configured filename
    // We don't modify request here; execution uses config info
    if let Err(e) = state.sender.send((id, req.clone())).await {
        let mut jobs = state.jobs.write().await;
        jobs.insert(id, JobState::Error(format!("queue error: {}", e)));
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to enqueue job"})),
        )
            .into_response();
    }

    (StatusCode::ACCEPTED, Json(IdResponse { id })).into_response()
}

async fn status_handler(State(state): State<AppState>, Path(id): Path<u64>) -> impl IntoResponse {
    let jobs = state.jobs.read().await;
    if let Some(st) = jobs.get(&id) {
        let body = match st {
            JobState::Queued => JobStatusResponse::Queued,
            JobState::Running => JobStatusResponse::Running,
            JobState::Completed(res) => JobStatusResponse::Completed {
                result: res.clone(),
            },
            JobState::Error(err) => JobStatusResponse::Error { error: err.clone() },
        };
        (StatusCode::OK, Json(body)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Job not found"})),
        )
            .into_response()
    }
}

async fn execute_request(req: &ExecuteRequest, state: &AppState) -> Result<ExecuteResponse> {
    let cfg = state
        .configs
        .get(&req.language)
        .ok_or_else(|| anyhow::anyhow!("Unknown language: {}", req.language))?
        .clone();

    let temp_dir = tempfile::tempdir()?;
    let work_dir = temp_dir.path().to_path_buf();

    // Always write using configured file_name so compilers/runtimes find it
    let source_path = work_dir.join(&cfg.file_name);
    tokio::fs::write(&source_path, &req.code).await?;

    // Compile if needed
    let mut compiled = false;
    if let Some(compile_command) = &cfg.compile_command {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(&["/C", compile_command]);
            c
        } else {
            Command::new(compile_command)
        };
        cmd.current_dir(&work_dir);
        cmd.args(&cfg.compile_args);
        let output = cmd.output().await?;
        if !output.status.success() {
            return Ok(ExecuteResponse {
                compiled: false,
                language: req.language.clone(),
                status: Some(ExecutionStatus::CompileError),
                message: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                results: vec![],
                total_duration_ms: 0,
            });
        }
        compiled = true;
    }

    let mut results = Vec::with_capacity(req.testcases.len());
    let mut total_duration_ms: u64 = 0;
    for tc in &req.testcases {
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd");
            c.args(&["/C", &cfg.run_command]);
            c
        } else {
            Command::new(&cfg.run_command)
        };
        cmd.current_dir(&work_dir);
        cmd.args(&cfg.run_args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn()?;
        let start = Instant::now();

        // Write stdin then close
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(tc.input.as_bytes()).await?;
        }

        // Capture stdout/stderr concurrently
        let mut out_pipe = child.stdout.take().unwrap();
        let mut err_pipe = child.stderr.take().unwrap();
        let out_handle = tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = out_pipe.read_to_end(&mut buf).await;
            buf
        });
        let err_handle = tokio::spawn(async move {
            let mut buf = Vec::new();
            let _ = err_pipe.read_to_end(&mut buf).await;
            buf
        });

        let timeout_ms = tc.timeout_ms.unwrap_or(5000);
        let mut timed_out = false;
        let status = tokio::select! {
            res = child.wait() => { res? }
            _ = time::sleep(time::Duration::from_millis(timeout_ms)) => {
                timed_out = true;
                let _ = child.kill().await; // Best-effort
                child.wait().await?
            }
        };

        let out_bytes = out_handle.await.unwrap_or_else(|_| Vec::new());
        let err_bytes = err_handle.await.unwrap_or_else(|_| Vec::new());
        let stdout = String::from_utf8_lossy(&out_bytes).to_string();
        let stderr = String::from_utf8_lossy(&err_bytes).to_string();
        let exit_code = status.code();
        let success = status.success();

        let duration_ms = start.elapsed().as_millis() as u64;
        total_duration_ms += duration_ms;

        let ok = success && !timed_out;
        let passed = match &tc.expected {
            Some(exp) => stdout == *exp,
            None => false,
        };

        results.push(CaseResult {
            id: tc.id,
            ok,
            passed,
            input: tc.input.clone(),
            expected: tc.expected.clone(),
            stdout,
            stderr,
            timed_out,
            duration_ms,
            memory_kb: 0,
            exit_code,
            term_signal: None,
        });
    }

    Ok(ExecuteResponse {
        compiled,
        language: req.language.clone(),
        status: Some(ExecutionStatus::Success),
        message: None,
        results,
        total_duration_ms,
    })
}
