@echo off
REM BuildIT Agent - Windows Defender Exclusion Setup
REM Run this as Administrator to add Windows Defender exclusion

echo ========================================
echo BuildIT Agent - Defender Exclusion Tool
echo ========================================
echo.

REM Check for admin privileges
net session >nul 2>&1
if %errorLevel% NEQ 0 (
    echo ERROR: This script must be run as Administrator
    echo.
    echo Right-click this file and select "Run as administrator"
    echo.
    pause
    exit /b 1
)

echo This script will add build-it-agent.exe to Windows Defender exclusions
echo.

REM Get the current directory
set SCRIPT_DIR=%~dp0
set EXE_PATH=%SCRIPT_DIR%..\target\release\build-it-agent.exe

REM Check if executable exists
if not exist "%EXE_PATH%" (
    echo ERROR: Executable not found at:
    echo %EXE_PATH%
    echo.
    echo Please build the project first:
    echo   cargo build --release --target x86_64-pc-windows-msvc
    echo.
    pause
    exit /b 1
)

echo Found executable: %EXE_PATH%
echo.
echo Adding exclusion to Windows Defender...
echo.

REM Add the exclusion using PowerShell
powershell -Command "Add-MpPreference -ExclusionPath '%EXE_PATH%'"

if %errorLevel% EQU 0 (
    echo.
    echo SUCCESS: Exclusion added!
    echo.
    echo Windows Defender will no longer block: %EXE_PATH%
    echo.
    echo You can now run the agent without interference.
    echo.
) else (
    echo.
    echo ERROR: Failed to add exclusion
    echo.
    echo Try manually adding the exclusion:
    echo 1. Open Windows Security
    echo 2. Go to Virus ^& threat protection
    echo 3. Click Manage settings
    echo 4. Scroll to Exclusions and click Add or remove exclusions
    echo 5. Add: %EXE_PATH%
    echo.
)

echo.
echo IMPORTANT NOTES:
echo ================
echo - This exclusion only works on THIS computer
echo - For distribution, consider code signing (see WINDOWS_DEFENDER_FIX.md)
echo - Submit to Microsoft as false positive: https://www.microsoft.com/en-us/wdsi/filesubmission
echo.

pause
