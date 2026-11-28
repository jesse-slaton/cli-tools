# Code signing policy

Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

## Scope

This policy covers the Windows release binaries built from this repository:

| Tool | Signed file | Shipped in |
|---|---|---|
| Path Commander | `pc.exe` | `path-commander-<version>-windows-x64.zip` on [GitHub Releases](https://github.com/jesse-slaton/cli-tools/releases) |

Nothing else is signed under this certificate. Third-party libraries are compiled into `pc.exe` from source by Cargo; no separately distributed upstream binaries are signed.

## How signed releases are built

- Every signed binary is built by the [Release workflow](.github/workflows/release.yml) on a GitHub-hosted runner, from a `vX.Y.Z` tag in this public repository.
- The workflow submits the unsigned `pc.exe` to SignPath through its GitHub trusted build system integration, which verifies that the artifact came from this repository, commit, and workflow run.
- Each release signing request is manually approved by an approver (below) before SignPath signs it.
- The binary's embedded product name and version are checked by SignPath against the release tag; a mismatch is rejected.
- Builds started manually (`workflow_dispatch`) or on a fork are never signed.

## Team roles

| Role | Members |
|---|---|
| Committers and reviewers | [Jesse Slaton (@jesse-slaton)](https://github.com/jesse-slaton) |
| Approvers | [Jesse Slaton (@jesse-slaton)](https://github.com/jesse-slaton) |

Contributions from anyone outside this list arrive as pull requests and are reviewed by a reviewer before merge. All members use multi-factor authentication on GitHub and SignPath.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it.

Apart from checking whether the folders on your PATH exist, which contacts the host of any UNC path (`\\server\share\...`) or mapped network drive on your PATH, the only network activity Path Commander performs is the remote management feature, which the user starts explicitly (`pc --remote COMPUTERNAME` or the connect dialog). It then reads and writes the PATH variables on that computer through the Windows Remote Registry service and checks path existence over administrative UNC shares (`\\COMPUTERNAME\C$\...`). No telemetry, update checks, or analytics are included.

## System changes

Path Commander edits the USER and MACHINE `PATH` environment variables, which is a system configuration change. It:

- shows a confirmation dialog before destructive operations,
- writes a timestamped backup before applying changes, and
- requires administrator privileges for MACHINE changes, and says so in the interface.

## Installation and uninstallation

Path Commander has no installer. To install, extract `pc.exe` from the release zip into a folder on your `PATH`.

To uninstall:

1. Delete `pc.exe`.
2. Optionally delete `%USERPROFILE%\.pc\`, which holds settings, themes, and PATH backups. Restore a backup first if you want to roll back changes Path Commander made.
3. If you added the install folder to `PATH` yourself, remove that entry.

## Reporting a problem

Report a signed file that you believe violates this policy to the maintainer through [SECURITY.md](SECURITY.md), or to SignPath at support@signpath.io.
