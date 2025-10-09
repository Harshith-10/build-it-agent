# Code Signing Script for BuildIT Agent
# Run this script as Administrator in PowerShell

param(
    [string]$CertificateFile = "",
    [string]$Password = "",
    [string]$ExePath = "target\release\build-it-agent.exe",
    [switch]$CreateSelfSigned = $false
)

Write-Host "BuildIT Agent Code Signing Utility" -ForegroundColor Cyan
Write-Host "===================================`n" -ForegroundColor Cyan

# Check if running as Administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: This script must be run as Administrator" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

# Check if executable exists
if (-not (Test-Path $ExePath)) {
    Write-Host "ERROR: Executable not found at: $ExePath" -ForegroundColor Red
    Write-Host "Please build the project first: cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor Yellow
    exit 1
}

# Option 1: Create Self-Signed Certificate
if ($CreateSelfSigned) {
    Write-Host "Creating self-signed code signing certificate..." -ForegroundColor Yellow
    
    $cert = New-SelfSignedCertificate `
        -Subject "CN=BuildIT Education Platform, O=BuildIT, C=US" `
        -Type CodeSigningCert `
        -CertStoreLocation Cert:\CurrentUser\My `
        -NotAfter (Get-Date).AddYears(3)
    
    Write-Host "Certificate created successfully!" -ForegroundColor Green
    Write-Host "Thumbprint: $($cert.Thumbprint)" -ForegroundColor Cyan
    
    # Export certificate
    $exportPath = "BuildIT-CodeSigning.cer"
    Export-Certificate -Cert $cert -FilePath $exportPath | Out-Null
    Write-Host "Certificate exported to: $exportPath" -ForegroundColor Green
    
    # Sign the executable
    Write-Host "`nSigning executable..." -ForegroundColor Yellow
    $signResult = Set-AuthenticodeSignature `
        -FilePath $ExePath `
        -Certificate $cert `
        -TimestampServer "http://timestamp.digicert.com" `
        -HashAlgorithm SHA256
    
    if ($signResult.Status -eq "Valid") {
        Write-Host "✓ Executable signed successfully!" -ForegroundColor Green
    } else {
        Write-Host "✗ Signing failed: $($signResult.StatusMessage)" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "`n⚠ NOTE: Self-signed certificates still trigger SmartScreen warnings" -ForegroundColor Yellow
    Write-Host "For production, obtain a certificate from a trusted CA (DigiCert, Sectigo, etc.)" -ForegroundColor Yellow
    
    exit 0
}

# Option 2: Sign with existing certificate
if ($CertificateFile -and (Test-Path $CertificateFile)) {
    Write-Host "Signing with certificate: $CertificateFile" -ForegroundColor Yellow
    
    # Check if SignTool is available
    $signTool = Get-Command "signtool.exe" -ErrorAction SilentlyContinue
    if (-not $signTool) {
        Write-Host "ERROR: SignTool not found. Please install Windows SDK" -ForegroundColor Red
        Write-Host "Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/" -ForegroundColor Yellow
        exit 1
    }
    
    # Build signtool command
    $signToolArgs = @(
        "sign",
        "/f", $CertificateFile,
        "/tr", "http://timestamp.digicert.com",
        "/td", "sha256",
        "/fd", "sha256"
    )
    
    if ($Password) {
        $signToolArgs += "/p"
        $signToolArgs += $Password
    }
    
    $signToolArgs += $ExePath
    
    Write-Host "Running: signtool.exe $signToolArgs" -ForegroundColor Cyan
    & signtool.exe $signToolArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Executable signed successfully!" -ForegroundColor Green
    } else {
        Write-Host "✗ Signing failed with exit code: $LASTEXITCODE" -ForegroundColor Red
        exit 1
    }
    
    exit 0
}

# No valid options provided - show help
Write-Host "Usage:" -ForegroundColor Yellow
Write-Host "  Create self-signed certificate and sign:" -ForegroundColor White
Write-Host "    .\scripts\sign-executable.ps1 -CreateSelfSigned`n" -ForegroundColor Cyan

Write-Host "  Sign with existing PFX certificate:" -ForegroundColor White
Write-Host "    .\scripts\sign-executable.ps1 -CertificateFile 'cert.pfx' -Password 'your-password'`n" -ForegroundColor Cyan

Write-Host "  Sign with custom executable path:" -ForegroundColor White
Write-Host "    .\scripts\sign-executable.ps1 -CreateSelfSigned -ExePath 'custom\path\to\build-it-agent.exe'`n" -ForegroundColor Cyan

Write-Host "For production releases, obtain a code signing certificate from:" -ForegroundColor Yellow
Write-Host "  - DigiCert: https://www.digicert.com/signing/code-signing-certificates" -ForegroundColor White
Write-Host "  - Sectigo: https://sectigo.com/ssl-certificates-tls/code-signing" -ForegroundColor White
Write-Host "  - GlobalSign: https://www.globalsign.com/en/code-signing-certificate" -ForegroundColor White
Write-Host "  - SSL.com: https://www.ssl.com/certificates/code-signing/" -ForegroundColor White
