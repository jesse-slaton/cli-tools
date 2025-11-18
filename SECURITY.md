# Security Policy

## Supported Versions

We release security updates for the following versions:

| Project | Version | Supported          |
| ------- | ------- | ------------------ |
| Path Commander | 0.6.x   | :white_check_mark: |
| Path Commander | < 0.6   | :x:                |

## Reporting a Vulnerability

We take security vulnerabilities seriously. If you discover a security issue in any of the CLI tools in this repository, please report it responsibly.

### How to Report

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities by:

1. **Using GitHub Security Advisories** (preferred)
   - Go to the [Security tab](https://github.com/jesse-slaton/cli-tools/security/advisories)
   - Click "Report a vulnerability"
   - Fill out the form with details

2. **Email** (alternative)
   - Send an email to the repository maintainer
   - Include "SECURITY" in the subject line
   - Provide detailed information about the vulnerability

### What to Include

When reporting a vulnerability, please include:

- **Description**: A clear description of the vulnerability
- **Impact**: What an attacker could do with this vulnerability
- **Steps to Reproduce**: Detailed steps to reproduce the issue
- **Affected Versions**: Which versions are affected
- **Suggested Fix**: If you have ideas for a fix (optional)
- **Proof of Concept**: Code or commands that demonstrate the issue (if applicable)

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Updates**: We will provide regular updates on our progress (at least every 7 days)
- **Timeline**: We aim to release a fix within 90 days for critical vulnerabilities
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

### Safe Harbor

We support safe harbor for security researchers who:

- Make a good faith effort to avoid privacy violations, data destruction, and service interruption
- Only interact with test accounts you own or with explicit permission from the account holder
- Do not exploit the vulnerability beyond the minimum necessary to demonstrate the issue
- Report vulnerabilities promptly
- Keep the vulnerability confidential until it has been resolved

We will not pursue legal action against researchers who follow these guidelines.

## Security Considerations for Path Commander

### Elevated Privileges

Path Commander can run with administrator privileges to modify MACHINE (system-wide) PATH variables. When running with elevated privileges:

- Only use trusted builds from official releases or build from verified source
- Review changes in the staged changes panel before applying
- Be cautious when connecting to remote computers
- Verify remote computer names before modifying their PATH variables

### Remote Computer Management

When using the remote computer management feature:

- Ensure you have proper authorization to modify PATH variables on remote computers
- Network credentials are handled by Windows authentication (WinRM/Remote Registry)
- Remote operations require appropriate permissions on the target computer
- Always verify you're connected to the correct computer before making changes

### Backup Files

Path Commander stores backups in `%LOCALAPPDATA%\PathCommander\backups\`:

- Backup files contain sensitive system configuration information
- Files are stored locally on your computer
- Ensure appropriate file system permissions on the backup directory
- Consider encrypting your user profile if storing sensitive data

## Known Security Limitations

- **Registry Access**: Direct registry modification requires careful handling - always review staged changes
- **No Input Sanitization Needed**: Path entries are stored as-is in the registry; Windows handles validation
- **Local Storage**: Backups are stored unencrypted in the user's local app data directory

## Security Best Practices

When using any CLI tools in this repository:

1. **Download from trusted sources**: Use official GitHub releases or build from source
2. **Verify checksums**: Check SHA256 hashes for downloaded binaries
3. **Run with least privilege**: Only use administrator privileges when necessary
4. **Review changes**: Always review staged changes before applying
5. **Keep backups**: Use the backup feature before making significant changes
6. **Stay updated**: Keep your tools updated to the latest version for security fixes

## Disclosure Policy

- We follow a **coordinated disclosure** policy
- Security issues will be disclosed publicly after a fix is available
- We will publish a security advisory with details of the vulnerability and the fix
- Critical vulnerabilities will be fast-tracked for release

## Contact

For security-related questions that are not vulnerabilities, you can:

- Open a regular GitHub issue with the `security` label
- Use the Discussions tab for general security questions

Thank you for helping keep our projects and users safe!
