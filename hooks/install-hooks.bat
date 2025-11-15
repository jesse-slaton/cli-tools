@echo off
REM Install Git hooks for the cli-tools repository
REM This is a wrapper that calls the PowerShell script

echo.
echo Installing Git hooks for cli-tools repository...
echo.

REM Get the directory where this script is located
set SCRIPT_DIR=%~dp0

REM Call the PowerShell script
powershell -ExecutionPolicy Bypass -File "%SCRIPT_DIR%install-hooks.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Installation failed!
    pause
    exit /b 1
)

echo.
pause
