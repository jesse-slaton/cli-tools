# Testing GitHub Actions Locally

You can test workflows locally before pushing to GitHub using [`act`](https://github.com/nektos/act).

## Installation

### Windows (via Chocolatey)
```powershell
choco install act-cli
```

### Windows (via Scoop)
```powershell
scoop install act
```

### macOS/Linux
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

## Basic Usage

### Test a specific workflow
```bash
# Test the CodeQL workflow
act push -W .github/workflows/codeql.yml

# Test the Rust security audit
act push -W .github/workflows/rust-security.yml

# Test the release workflow
act push -W .github/workflows/release.yml
```

### Test a specific job
```bash
# Test only the dependency check job
act -j dependency-check
```

### Dry run (see what would happen)
```bash
act -n
```

## Common Options

```bash
# Use a specific runner image
act -P ubuntu-latest=catthehacker/ubuntu:act-latest

# Set secrets
act -s GITHUB_TOKEN=your_token_here

# Verbose output for debugging
act -v

# List all workflows that would run
act -l
```

## Docker Requirement

`act` requires Docker to be installed and running since it uses Docker containers to simulate GitHub runners.

### Install Docker Desktop
- Windows: https://docs.docker.com/desktop/install/windows-install/
- macOS: https://docs.docker.com/desktop/install/mac-install/
- Linux: https://docs.docker.com/desktop/install/linux-install/

## Limitations

`act` has some limitations compared to actual GitHub Actions:

1. **Windows-only workflows** - Won't work locally on macOS/Linux
2. **Secrets** - Must be provided manually with `-s` flag
3. **GitHub-specific features** - Some GitHub integrations may not work exactly the same
4. **Performance** - May be slower than GitHub's hosted runners

## Quick Iteration Workflow

For iterating on workflow files:

1. **Edit the workflow file** locally
2. **Test with act**:
   ```bash
   act push -W .github/workflows/rust-security.yml -j dependency-check
   ```
3. **Fix issues** based on output
4. **Repeat** steps 2-3 until it works
5. **Commit and push** to GitHub

This saves you from creating dozens of commits just to test workflow syntax.

## Example: Testing Rust Security Workflow

```bash
# Navigate to repo
cd D:/Dev/MyGithubRepos/cli-tools

# Test just the dependency check job
act push -W .github/workflows/rust-security.yml -j dependency-check

# Or test the full workflow
act push -W .github/workflows/rust-security.yml
```

## Troubleshooting

### "Cannot connect to Docker daemon"
Make sure Docker Desktop is running.

### "Image not found"
Pull the default act images:
```bash
docker pull catthehacker/ubuntu:act-latest
```

### Workflow doesn't trigger
Check the `on:` triggers in your workflow match what act is simulating (usually `push`).

## References

- [`act` GitHub Repository](https://github.com/nektos/act)
- [`act` Documentation](https://nektosact.com/)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/)
