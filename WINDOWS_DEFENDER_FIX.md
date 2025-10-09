# Windows Defender False Positive Solutions

## Issue
Windows Defender may flag `build-it-agent.exe` as `Trojan:Win32/Sabsik.FL.A!ml` or similar threats. This is a **false positive** because the agent legitimately:
- Monitors running processes (for exam integrity)
- Spawns child processes (for code execution)
- Uses network capabilities (local web server)
- Enumerates windows (for detecting forbidden apps)

These behaviors are legitimate for an educational assessment tool but trigger heuristic malware detection.

## Solutions Implemented

### 1. Version Info & Metadata (Already Applied)
The build now embeds Windows version information and an application manifest:
- Company name, product description, copyright info
- Application manifest declaring purpose and permissions
- Build resources in `resources/windows/`

This helps Windows Defender identify the application as legitimate.

### 2. Rebuild After Changes
After pulling these changes, rebuild for Windows:

```bash
# For Windows x86_64
cargo build --release --target x86_64-pc-windows-msvc

# For Windows x86_64 with GNU toolchain
cargo build --release --target x86_64-pc-windows-gnu
```

### 3. Code Signing (Recommended for Production)

For production deployments, **code signing** is the most effective solution:

#### Option A: Self-Signed Certificate (Development/Testing)
```powershell
# Create a self-signed certificate (PowerShell as Administrator)
$cert = New-SelfSignedCertificate -DnsName "BuildIT" -Type CodeSigning -CertStoreLocation Cert:\CurrentUser\My

# Export the certificate
Export-Certificate -Cert $cert -FilePath "BuildIT.cer"

# Sign the executable
$certPath = Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert
Set-AuthenticodeSignature -FilePath "target\release\build-it-agent.exe" -Certificate $certPath -TimestampServer "http://timestamp.digicert.com"
```

**Note:** Self-signed certificates still trigger SmartScreen warnings and may not fully resolve Windows Defender issues.

#### Option B: Commercial Certificate (Production)
For production, obtain a code signing certificate from a trusted Certificate Authority:

**Trusted Certificate Authorities:**
- DigiCert (https://www.digicert.com/signing/code-signing-certificates)
- Sectigo (https://sectigo.com/ssl-certificates-tls/code-signing)
- GlobalSign (https://www.globalsign.com/en/code-signing-certificate)
- SSL.com (https://www.ssl.com/certificates/code-signing/)

**Typical Cost:** $100-$500/year

**Signing Process:**
```bash
# Using SignTool (from Windows SDK)
signtool sign /f your-certificate.pfx /p your-password /tr http://timestamp.digicert.com /td sha256 /fd sha256 target\release\build-it-agent.exe
```

**Benefits of Commercial Certificates:**
- Eliminates SmartScreen warnings
- Significantly reduces false positives
- Builds user trust
- Required for widespread distribution

### 4. Submit False Positive to Microsoft

If rebuilding doesn't resolve the issue, submit the binary to Microsoft:

1. Visit: https://www.microsoft.com/en-us/wdsi/filesubmission
2. Select "Submit a file for malware analysis"
3. Upload: `build-it-agent.exe`
4. Provide details:
   - **Application Name:** BuildIT Agent
   - **Purpose:** Educational code execution and process monitoring for secure online assessments
   - **Description:** Legitimate educational software that monitors processes and executes student code in sandboxed environments during exams
5. Microsoft typically responds within 24-48 hours

### 5. Add Exclusion (Temporary Workaround)

For testing/development, add a Windows Defender exclusion:

```powershell
# PowerShell as Administrator
Add-MpPreference -ExclusionPath "C:\path\to\build-it-agent.exe"

# Or add folder exclusion
Add-MpPreference -ExclusionPath "C:\path\to\project\target\release"
```

**For End Users:**
1. Open Windows Security
2. Go to "Virus & threat protection"
3. Click "Manage settings" under "Virus & threat protection settings"
4. Scroll down to "Exclusions" and click "Add or remove exclusions"
5. Add `build-it-agent.exe` or its folder

**⚠️ Warning:** Only use exclusions for trusted files. This bypasses security protection.

### 6. Distribute Through Microsoft Store

Publishing through the Microsoft Store:
- Binaries are automatically trusted
- No code signing certificate needed
- Microsoft performs security review
- Significantly reduces false positives

## Why This Happens

Modern antivirus software uses heuristic analysis to detect malware. The BuildIT Agent exhibits behaviors similar to some malware:

| Behavior | Legitimate Purpose | Why Flagged |
|----------|-------------------|-------------|
| Process monitoring | Detect forbidden apps during exams | Similar to spyware |
| Process spawning | Execute student code | Similar to droppers |
| Network server | Receive execution requests | Similar to command & control |
| Window enumeration | Detect topmost windows | Similar to screen capture malware |

## Verification

After rebuilding, verify the embedded metadata:

```powershell
# PowerShell
Get-ItemProperty "target\release\build-it-agent.exe" | Select-Object VersionInfo
```

You should see:
- Company Name: BuildIT Education Platform
- Product Name: BuildIT Agent
- File Description: Educational code execution agent

## Additional Resources

- [Microsoft Security Intelligence](https://www.microsoft.com/en-us/wdsi)
- [Code Signing Best Practices](https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools)
- [Windows Defender Submission Portal](https://www.microsoft.com/en-us/wdsi/filesubmission)

## Support

If issues persist after trying these solutions, please:
1. Check antivirus logs for specific detection details
2. Share the exact threat name from Windows Defender
3. Try rebuilding with the latest changes
4. Consider submitting to Microsoft as a false positive
