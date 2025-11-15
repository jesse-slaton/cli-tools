# Install Git hooks for the cli-tools repository
# This script copies hook templates from hooks/ to .git/hooks/

$ErrorActionPreference = "Stop"

Write-Host "`nInstalling Git hooks for cli-tools repository...`n" -ForegroundColor Blue

# Get the repository root directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Split-Path -Parent $ScriptDir
$HooksDir = Join-Path $RepoRoot "hooks"
$GitHooksDir = Join-Path $RepoRoot ".git\hooks"

# Check if we're in a git repository
if (-not (Test-Path $GitHooksDir)) {
    Write-Host "Error: Not in a git repository or .git/hooks directory not found" -ForegroundColor Yellow
    exit 1
}

# Check if hooks directory exists
if (-not (Test-Path $HooksDir)) {
    Write-Host "Error: hooks/ directory not found" -ForegroundColor Yellow
    exit 1
}

# Install pre-commit hook
$PreCommitSource = Join-Path $HooksDir "pre-commit"
$PreCommitDest = Join-Path $GitHooksDir "pre-commit"

if (Test-Path $PreCommitSource) {
    Write-Host "→ Installing pre-commit hook..." -ForegroundColor Blue
    Copy-Item -Path $PreCommitSource -Destination $PreCommitDest -Force

    # On Windows with Git for Windows, the hook should already be executable
    # But we can verify the file was copied
    if (Test-Path $PreCommitDest) {
        Write-Host "  ✓ pre-commit hook installed" -ForegroundColor Green
    } else {
        Write-Host "  ! Failed to install pre-commit hook" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ! pre-commit hook template not found" -ForegroundColor Yellow
}

Write-Host "`n✓ Git hooks installation complete!`n" -ForegroundColor Green

Write-Host "The following checks will run before each commit:"
Write-Host "  • Code formatting (cargo fmt)"
Write-Host "  • Linting (cargo clippy)"
Write-Host "  • Build verification (cargo check)"
Write-Host "  • Unit tests (cargo test)"
Write-Host "  • Documentation build (cargo doc)"
Write-Host "  • Security audit (cargo audit, if installed)"

Write-Host "`nNote: " -ForegroundColor Blue -NoNewline
Write-Host "You can bypass hooks with: " -NoNewline
Write-Host "git commit --no-verify`n" -ForegroundColor Yellow
