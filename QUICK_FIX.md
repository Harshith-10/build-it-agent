# Windows Defender Quick Fix Guide

## 🚨 Issue
Windows Defender blocks `build-it-agent.exe` as malware (FALSE POSITIVE)

## ✅ Quick Solutions (Choose One)

### 1️⃣ IMMEDIATE FIX: Add Windows Defender Exclusion
**⚠️ Use this to get running quickly, but it's not ideal for distribution**

**Via PowerShell (Run as Administrator):**
```powershell
Add-MpPreference -ExclusionPath "C:\full\path\to\build-it-agent.exe"
```

**Via Windows Security GUI:**
1. Open **Windows Security** → **Virus & threat protection**
2. Click **Manage settings**
3. Scroll to **Exclusions** → Click **Add or remove exclusions**
4. Click **Add an exclusion** → Choose **File**
5. Select `build-it-agent.exe`

---

### 2️⃣ REBUILD WITH METADATA (Recommended)
**This adds version info to help Windows Defender recognize it as legitimate**

```bash
# Pull the latest changes (includes build.rs and Windows resources)
git pull

# Rebuild for Windows
cargo build --release --target x86_64-pc-windows-msvc
```

The new build includes:
- ✓ Company name and product information
- ✓ Application description and purpose
- ✓ Windows manifest with proper permissions
- ✓ Version information resource

---

### 3️⃣ SUBMIT FALSE POSITIVE TO MICROSOFT
**Helps everyone who downloads your app**

1. Go to: https://www.microsoft.com/en-us/wdsi/filesubmission
2. Select **"Submit a file for malware analysis"**
3. Upload: `build-it-agent.exe`
4. Use this description:

```
This is BuildIT Agent, a legitimate educational code execution and 
monitoring tool used for secure online assessments. It monitors processes 
and executes student code during supervised exams. Detection as malware 
is a FALSE POSITIVE.

The application:
- Only runs on localhost (127.0.0.1)
- Does not communicate externally
- Is used with user consent during exams
- Contains embedded version information verifying its purpose
```

5. Microsoft typically responds in 24-48 hours

---

### 4️⃣ CODE SIGNING (Production/Distribution)
**The ultimate solution - eliminates false positives**

**For Development/Testing (Self-Signed):**
```powershell
# Run as Administrator
.\scripts\sign-executable.ps1 -CreateSelfSigned
```

**For Production (Commercial Certificate - ~$200/year):**
1. Purchase certificate from:
   - [DigiCert](https://www.digicert.com/signing/code-signing-certificates)
   - [Sectigo](https://sectigo.com/ssl-certificates-tls/code-signing)
   - [GlobalSign](https://www.globalsign.com/en/code-signing-certificate)

2. Sign your executable:
```powershell
.\scripts\sign-executable.ps1 -CertificateFile "your-cert.pfx" -Password "your-password"
```

---

## 🔍 Why Does This Happen?

BuildIT Agent is flagged because it legitimately:
- ✓ Monitors running processes (to detect forbidden apps during exams)
- ✓ Spawns child processes (to execute student code)
- ✓ Runs a local web server (to receive execution requests)
- ✓ Enumerates windows (to find unauthorized applications)

These are **legitimate features** for an educational assessment tool, but they look similar to malware behaviors to heuristic scanners.

---

## 📊 Solution Comparison

| Solution | Time | Cost | Effectiveness | Best For |
|----------|------|------|---------------|----------|
| Exclusion | 2 min | Free | ⭐⭐ (Local only) | Quick testing |
| Rebuild | 5 min | Free | ⭐⭐⭐ (Improved) | Development |
| MS Submission | 1-2 days | Free | ⭐⭐⭐⭐ (Helps others) | Distribution |
| Code Signing | 1 hour | $200/yr | ⭐⭐⭐⭐⭐ (Best) | Production |

---

## 📖 Full Documentation
See `WINDOWS_DEFENDER_FIX.md` for detailed information and troubleshooting.

---

## ✉️ Still Having Issues?

1. Check Windows Defender logs for the exact detection name
2. Verify you've rebuilt after pulling the latest changes
3. Try submitting to Microsoft if rebuilding doesn't help
4. Consider code signing for production releases
