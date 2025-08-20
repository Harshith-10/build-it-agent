use anyhow::Result;
use axum::{extract::Query, response::IntoResponse, routing::get, Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, net::SocketAddr, sync::Arc};
use sysinfo::System;

#[cfg(windows)]
use std::sync::Mutex;
#[cfg(windows)]
use winapi::{
    shared::windef::HWND,
    um::winuser::{
        EnumWindows, GetWindowLongW, GetWindowThreadProcessId, IsWindowVisible, GWL_EXSTYLE,
        WS_EX_TOPMOST,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusResponse {
    pub timestamp: String,
    pub forbidden_processes: Vec<String>,
    pub platform: String,
}

#[derive(Deserialize)]
pub struct StatusQuery {
    #[serde(default)]
    pub include_topmost: bool,
}

pub fn get_default_forbidden_list() -> Vec<String> {
    let mut forbidden = Vec::new();

    // Cross-platform applications
    forbidden.extend([
        "Code.exe".to_string(),
        "code".to_string(), // VS Code
        "devenv.exe".to_string(),
        "devenv".to_string(), // Visual Studio
        "idea64.exe".to_string(),
        "idea".to_string(),
        "IntelliJ IDEA".to_string(), // IntelliJ IDEA
        "PyCharm".to_string(),
        "pycharm".to_string(), // PyCharm
        "eclipse".to_string(),
        "Eclipse".to_string(), // Eclipse
        "atom".to_string(),
        "Atom".to_string(), // Atom
        "sublime_text".to_string(),
        "Sublime Text".to_string(), // Sublime Text
        "notepad++.exe".to_string(),
        "Notepad++".to_string(), // Notepad++
        "vim".to_string(),
        "nvim".to_string(),
        "emacs".to_string(), // Terminal editors
        "AutoHotkey.exe".to_string(),
        "autohotkey".to_string(), // AutoHotkey
        "obs64.exe".to_string(),
        "obs".to_string(),
        "OBS Studio".to_string(), // OBS Studio
        "PowerToys.exe".to_string(),
        "PowerToys".to_string(), // PowerToys
        "ollama".to_string(),
        "Ollama".to_string(), // Ollama
        "docker".to_string(),
        "Docker Desktop".to_string(), // Docker
        "virtualbox".to_string(),
        "VirtualBox".to_string(), // VirtualBox
        "vmware".to_string(),
        "VMware".to_string(), // VMware
        "wireshark".to_string(),
        "Wireshark".to_string(), // Wireshark
        "fiddler".to_string(),
        "Fiddler".to_string(), // Fiddler
        "burp".to_string(),
        "Burp Suite".to_string(), // Burp Suite
        "ida".to_string(),
        "IDA Pro".to_string(), // IDA Pro
        "ghidra".to_string(),
        "Ghidra".to_string(), // Ghidra
        "x64dbg".to_string(),
        "x32dbg".to_string(), // x64dbg/x32dbg
        "ollydbg".to_string(),
        "OllyDbg".to_string(), // OllyDbg
        "cheat engine".to_string(),
        "Cheat Engine".to_string(), // Cheat Engine
        "process hacker".to_string(),
        "Process Hacker".to_string(), // Process Hacker
        "process monitor".to_string(),
        "Process Monitor".to_string(), // Process Monitor
        "autoruns".to_string(),
        "Autoruns".to_string(), // Autoruns
        "regshot".to_string(),
        "Regshot".to_string(), // Regshot
    ]);

    // Windows-specific
    if cfg!(windows) {
        forbidden.extend([
            "copilot.exe".to_string(),                    // Copilot
            "Copilot".to_string(),
            "mstsc.exe".to_string(),                      // Remote Desktop
            "TeamViewer.exe".to_string(),                 // TeamViewer
            "anydesk.exe".to_string(),                    // AnyDesk
            "chrome_remote_desktop_host.exe".to_string(), // Chrome Remote Desktop
            "LogMeIn.exe".to_string(),                    // LogMeIn
            "ammyy.exe".to_string(),                      // Ammyy Admin
            "radmin.exe".to_string(),                     // Radmin
            "dwservice.exe".to_string(),                  // DWService
            "supremo.exe".to_string(),                    // SupRemo
            "ultraviewer.exe".to_string(),                // UltraViewer
            "wsl.exe".to_string(),
            "Windows Subsystem for Linux".to_string(),    // WSL
        ]);
    }

    // macOS-specific
    if cfg!(target_os = "macos") {
        forbidden.extend([
            "Screen Sharing".to_string(),
            "Remote Desktop Scanner".to_string(),
            "Apple Remote Desktop".to_string(),
            "TeamViewer".to_string(),
            "AnyDesk".to_string(),
            "LogMeIn".to_string(),
            "Splashtop Business".to_string(),
            "Chrome Remote Desktop".to_string(),
            "VNC Viewer".to_string(),
            "Jump Desktop".to_string(),
            "Microsoft Remote Desktop".to_string(),
            "Parallels Desktop".to_string(),
            "VMware Fusion".to_string(),
            "UTM".to_string(),
        ]);
    }

    // Linux-specific
    if cfg!(target_os = "linux") {
        forbidden.extend([
            "teamviewer".to_string(),
            "anydesk".to_string(),
            "remmina".to_string(),
            "vinagre".to_string(),
            "krdc".to_string(),
            "xfreerdp".to_string(),
            "rdesktop".to_string(),
            "vnc".to_string(),
            "x11vnc".to_string(),
            "tightvnc".to_string(),
            "tigervnc".to_string(),
            "chrome-remote-desktop".to_string(),
            "nomachine".to_string(),
            "realvnc".to_string(),
            "ultravnc".to_string(),
            "qemu".to_string(),
            "virtualbox".to_string(),
            "vmware".to_string(),
            "kvm".to_string(),
            "gnome-boxes".to_string(),
        ]);
    }

    forbidden
}

#[cfg(windows)]
fn enumerate_topmost_processes() -> Vec<String> {
    let process_names = Mutex::new(Vec::<String>::new());

    extern "system" fn enum_callback(hwnd: HWND, lparam: isize) -> i32 {
        unsafe {
            let ptr = lparam as *const Mutex<Vec<String>>;
            let mutex: &Mutex<Vec<String>> = &*ptr;

            if IsWindowVisible(hwnd) == 0 {
                return 1; // Continue enumeration
            }

            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            if (ex_style as u32) & (WS_EX_TOPMOST as u32) != 0 {
                let mut pid: u32 = 0;
                GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);

                // Get process name from sysinfo
                let mut sys = System::new_all();
                sys.refresh_processes();

                if let Some(process) = sys.process(sysinfo::Pid::from_u32(pid)) {
                    let mut names = mutex.lock().unwrap();
                    names.push(process.name().to_string());
                }
            }
            1 // Continue enumeration
        }
    }

    let ptr = &process_names as *const _ as isize;
    unsafe {
        EnumWindows(Some(enum_callback), ptr);
    }

    process_names.into_inner().unwrap()
}

#[cfg(not(windows))]
fn enumerate_topmost_processes() -> Vec<String> {
    // On non-Windows platforms, we can't easily detect topmost windows
    Vec::new()
}

pub fn detect_forbidden_processes(forbidden_list: &[String], include_topmost: bool) -> Vec<String> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut detected = HashSet::new();

    // Get all running process names
    let mut all_processes = Vec::new();
    for (_pid, process) in sys.processes() {
        all_processes.push(process.name().to_string());
    }

    // Add topmost processes on Windows if requested
    if include_topmost {
        all_processes.extend(enumerate_topmost_processes());
    }

    // Check for forbidden processes (case-insensitive substring match)
    for forbidden in forbidden_list {
        let forbidden_lower = forbidden.to_lowercase();
        for process_name in &all_processes {
            let process_lower = process_name.to_lowercase();
            if process_lower.contains(&forbidden_lower) {
                detected.insert(process_name.clone());
            }
        }
    }

    let mut result: Vec<String> = detected.into_iter().collect();
    result.sort();
    result
}

pub fn build_app(forbidden_list: Arc<Vec<String>>) -> Router {
    Router::new().route(
        "/status",
        get({
            let forbidden = forbidden_list.clone();
            move |query| status_handler(query, forbidden)
        }),
    )
}

async fn status_handler(
    Query(params): Query<StatusQuery>,
    forbidden_list: Arc<Vec<String>>,
) -> impl IntoResponse {
    let platform = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };

    let forbidden_processes = detect_forbidden_processes(&forbidden_list, params.include_topmost);

    let response = StatusResponse {
        timestamp: Utc::now().to_rfc3339(),
        forbidden_processes,
        platform: platform.to_string(),
    };

    Json(response)
}

pub async fn run() -> Result<()> {
    println!("Starting cross-platform process monitor...");

    let forbidden_list = Arc::new(get_default_forbidden_list());

    println!("Checking for {} known forbidden processes", forbidden_list.len());
    println!(
        "Platform: {}",
        if cfg!(windows) {
            "Windows"
        } else if cfg!(target_os = "macos") {
            "macOS"
        } else if cfg!(target_os = "linux") {
            "Linux"
        } else {
            "Unknown"
        }
    );

    let app = build_app(forbidden_list.clone());
    let addr = SocketAddr::from(([127, 0, 0, 1], 8765));
    println!("Process monitor listening on http://{}", addr);
    println!("Try: curl http://localhost:8765/status");
    println!("With topmost detection (Windows only): curl 'http://localhost:8765/status?include_topmost=true'");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
