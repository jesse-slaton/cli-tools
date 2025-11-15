#!/usr/bin/env pwsh

Write-Host "Building Path Commander (pc.exe)..." -ForegroundColor Cyan
Write-Host ""

# Build release version
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "Build successful!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "Binary location: " -NoNewline
    Write-Host "target\release\pc.exe" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To install, copy to a directory in your PATH:"
    Write-Host "  copy target\release\pc.exe C:\Windows\System32\" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Or run directly:"
    Write-Host "  .\target\release\pc.exe" -ForegroundColor Gray
    Write-Host ""

    # Get file size
    $exePath = ".\target\release\pc.exe"
    if (Test-Path $exePath) {
        $fileSize = (Get-Item $exePath).Length
        $fileSizeMB = [math]::Round($fileSize / 1MB, 2)
        Write-Host "Binary size: $fileSizeMB MB" -ForegroundColor Cyan
    }
} else {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "Build failed!" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    exit 1
}
