@echo off
echo Building Path Commander (pc.exe)...
echo.

REM Build release version
cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ========================================
    echo Build successful!
    echo ========================================
    echo.
    echo Binary location: target\release\pc.exe
    echo.
    echo To install, copy to a directory in your PATH:
    echo   copy target\release\pc.exe C:\Windows\System32\
    echo.
    echo Or run directly:
    echo   target\release\pc.exe
    echo.
) else (
    echo.
    echo ========================================
    echo Build failed!
    echo ========================================
    echo.
)

pause
