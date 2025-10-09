# BuildIT Agent

A secure code execution and process monitoring agent for educational online assessments.

## 🛡️ Windows Defender False Positive Issue

**If Windows Defender is blocking the executable, see:**
- **[QUICK_FIX.md](QUICK_FIX.md)** - Fast solutions to get running immediately
- **[WINDOWS_DEFENDER_FIX.md](WINDOWS_DEFENDER_FIX.md)** - Detailed explanation and long-term solutions

**TL;DR:** This is a **false positive**. The agent legitimately monitors processes and executes code for exam integrity. 

### Quick Fix (2 minutes)
```powershell
# PowerShell as Administrator
Add-MpPreference -ExclusionPath "C:\path\to\build-it-agent.exe"
```

### Better Fix (5 minutes)
```bash
# Rebuild with embedded version information
cargo build --release --target x86_64-pc-windows-msvc
```

## Features

- **Multi-language Code Execution**: Supports Python, Java, C++, JavaScript, Rust, Go, and more
- **Process Monitoring**: Detects forbidden applications during assessments
- **RESTful API**: Local web server for code submission and execution
- **Secure Sandboxing**: Timeout controls and isolated execution
- **Cross-platform**: Windows, macOS, and Linux support

## Building

### Prerequisites
- Rust 1.70 or later
- For Windows builds: MSVC Build Tools or MinGW-w64

### Build Instructions

```bash
# Standard build
cargo build --release

# Windows-specific (includes version info and manifest)
cargo build --release --target x86_64-pc-windows-msvc

# Cross-compile for Windows from Linux
cargo build --release --target x86_64-pc-windows-gnu
```

### Building with Code Signing (Windows)

For production releases to avoid false positives:

```powershell
# Option 1: Self-signed (development/testing)
.\scripts\sign-executable.ps1 -CreateSelfSigned

# Option 2: Commercial certificate (production)
.\scripts\sign-executable.ps1 -CertificateFile "cert.pfx" -Password "pwd"
```

See [WINDOWS_DEFENDER_FIX.md](WINDOWS_DEFENDER_FIX.md) for certificate purchasing options.

## Usage

Start the agent:
```bash
./build-it-agent
```

The agent runs two services:
- **Executor API**: `http://localhost:8910` - Code execution endpoints
- **Monitor API**: `http://localhost:8911` - Process monitoring endpoints

### API Endpoints

**Executor (Port 8910):**
- `GET /health` - Health check
- `GET /languages` - List available languages
- `POST /execute` - Submit code for execution
- `GET /status/:id` - Check execution status

**Monitor (Port 8911):**
- `GET /status` - Get forbidden process list
- `DELETE /forbidden` - Kill forbidden processes (requires confirmation)

### Example: Execute Python Code

```bash
curl -X POST http://localhost:8910/execute \
  -H "Content-Type: application/json" \
  -d '{
    "language": "python",
    "code": "print(\"Hello, World!\")",
    "testCases": []
  }'
```

## Configuration

### Supported Languages

The agent auto-detects installed compilers/interpreters:
- Python, Java, C, C++, C#, JavaScript (Node.js), TypeScript
- Rust, Go, Ruby, PHP, Swift, Kotlin
- And more...

### Forbidden Processes

Default forbidden applications include:
- IDEs (VS Code, IntelliJ, PyCharm, etc.)
- Screen capture tools (OBS)
- AI assistants (Ollama)
- Automation tools (AutoHotkey, PowerToys)

See `src/monitor.rs` for the full list.

## Development

### Project Structure
```
build-it-agent/
├── src/
│   ├── main.rs           # Entry point
│   ├── executor.rs       # Code execution service
│   ├── monitor.rs        # Process monitoring service
│   ├── language.rs       # Language detection & configs
│   └── types.rs          # Shared types
├── resources/
│   └── windows/          # Windows resources (version info, manifest)
├── scripts/
│   └── sign-executable.ps1  # Code signing utility
└── tests/
    └── integration_tests.rs
```

### Running Tests
```bash
cargo test
```

### Development Build
```bash
cargo build
```

## Deployment

### For Windows Users

1. **Unsigned Binary**: Users will need to add Windows Defender exclusion (see QUICK_FIX.md)
2. **Signed Binary**: Purchase code signing certificate (~$200/year) and sign with:
   ```powershell
   .\scripts\sign-executable.ps1 -CertificateFile "cert.pfx" -Password "pwd"
   ```
3. **Microsoft Store**: Publish through Microsoft Store for automatic trust

### GitHub Actions

Automated builds and releases are configured in `.github/workflows/release.yml`

The workflow automatically:
- Builds binaries for all platforms (Linux, Windows, macOS - x86_64 and ARM64)
- Signs Windows binaries (if certificate secrets are configured)
- Creates GitHub releases on version tags (e.g., `v1.0.0`)
- Generates checksums for all binaries

To enable code signing in CI/CD:
1. Obtain a code signing certificate (.pfx file)
2. Convert to base64: `[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))`
3. Add GitHub Secrets:
   - `CERTIFICATE_BASE64`: Base64-encoded certificate
   - `CERTIFICATE_PASSWORD`: Certificate password

To create a release:
```bash
git tag v1.0.0
git push origin v1.0.0
```

## Security Considerations

- The agent requires **administrator privileges** on Windows for comprehensive process monitoring
- All code execution is **sandboxed** with timeout controls
- The web server **only listens on localhost** (127.0.0.1) - no external network access
- Process monitoring is **read-only** - termination requires explicit confirmation
- Temporary files are automatically cleaned up after execution

## License

[Your License Here]

## Support

For Windows Defender issues: See [WINDOWS_DEFENDER_FIX.md](WINDOWS_DEFENDER_FIX.md)

For other issues: [Open an issue on GitHub](https://github.com/Harshith-10/build-it-agent/issues)

## Why This Triggers Antivirus

BuildIT Agent performs legitimate functions that appear similar to malware:

| Feature | Legitimate Purpose | Why Flagged |
|---------|-------------------|-------------|
| Process monitoring | Detect forbidden apps | Similar to spyware |
| Process spawning | Execute student code | Similar to malware droppers |
| Network server | Receive code requests | Similar to C&C servers |
| Window enumeration | Find unauthorized apps | Similar to screen capture tools |

**This is a FALSE POSITIVE.** The application is designed for educational assessments and operates with full user knowledge and consent.

---

**⚠️ Important**: Do not close the agent window during an active examination, as this will terminate the service and may invalidate your assessment.
