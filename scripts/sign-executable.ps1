# Code Signing Script for BuildIT Agent
# Run this script as Administrator in PowerShell

param(
    [string]$ExePath = "target\release\build-it-agent.exe"
)

Write-Host "BuildIT Agent Code Signing Utility" -ForegroundColor Cyan
Write-Host "===================================`n" -ForegroundColor Cyan

# Check if running as Administrator (skip in CI environments)
$isCI = $env:CI -eq "true" -or $env:GITHUB_ACTIONS -eq "true"
if (-not $isCI) {
    $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    if (-not $isAdmin) {
        Write-Host "ERROR: This script must be run as Administrator" -ForegroundColor Red
        Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
        exit 1
    }
} else {
    Write-Host "Running in CI environment - skipping admin check" -ForegroundColor Yellow
}

# Check if executable exists
if (-not (Test-Path $ExePath)) {
    Write-Host "ERROR: Executable not found at: $ExePath" -ForegroundColor Red
    Write-Host "Please build the project first: cargo build --release --target x86_64-pc-windows-msvc" -ForegroundColor Yellow
    exit 1
}

Write-Host "Creating self-signed code signing certificate..." -ForegroundColor Yellow
    
$cert = New-SelfSignedCertificate `
    -Subject "CN=Institute of Aeronautical Engineering, O=BuildIT, C=IN" `
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
    
if ($signResult.Status -eq "Valid" -or $signResult.Status -eq "UnknownError") {
    Write-Host "[SUCCESS] Executable signed successfully!" -ForegroundColor Green
    Write-Host "  Status: $($signResult.Status)" -ForegroundColor Cyan
} else {
    Write-Host "[FAILED] Signing failed" -ForegroundColor Red
    Write-Host "  Status: $($signResult.Status)" -ForegroundColor Yellow
    Write-Host "  Message: $($signResult.StatusMessage)" -ForegroundColor Yellow
    exit 1
}
    
Write-Host "`nNOTE: Self-signed certificates still trigger SmartScreen warnings" -ForegroundColor Yellow
    
exit 0
