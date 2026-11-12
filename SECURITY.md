# Security policy

## Reporting a vulnerability

Please do not publish an unpatched vulnerability with private system logs, device serial numbers, MAC addresses, hostnames or other personal data in a public issue. Contact the project maintainer through the security contact listed in the GitHub repository profile and provide a minimal reproduction with identifying data removed.

## Security design

CrossDiag is local-first. Its frontend invokes named Tauri commands; it must not be allowed to select arbitrary executables or submit free-form shell commands. Native command adapters must validate arguments, use bounded execution time and output, and avoid collecting unnecessary identifiers. Privileged or destructive operations require a separate review, clear explanation and explicit user approval.

Diagnostic reports can contain sensitive information such as MAC addresses, device paths, hostnames, event text and software inventory. Review and redact a report before sharing it.


## Release signing and updates

The Tauri updater requires signed artifacts. The updater public key is a repository variable and is embedded in release builds; the signing private key and its password must be stored only in GitHub Actions Secrets. Never commit a private signing key, generated private key file, access token or signing password. Report suspected key exposure privately and treat it as a release-security incident.

The in-app update panel is disabled in local development/browser builds. Release builds verify signed update artifacts and require user approval before download and installation. Windows Authenticode signing is a separate control and is not yet configured by the project.

## Local security checks

Before opening a pull request, run `npm run test:security:source`. This checks secret-file ignore rules, updater build gating, Tauri capability scope, WebView security configuration, the release-signing secret wiring and the source tree for PEM private-key blocks. CI separately runs npm/Cargo dependency audits and CodeQL. These checks reduce risk but do not replace review of the actual dependency advisories, generated installers or runtime behavior.


## Local persistence and command output

Storage scan history, technician cases and schedule preferences are stored in the local SQLite database in the app-data directory. The database is not encrypted by CrossDiag; protect the operating-system account and redact exported reports before sharing them. Older JSON storage history is retained as a local migration backup. External process output is bounded to 8 MiB and each process receives a fixed timeout. Native commands must continue to use fixed executable names, validated arguments and explicit allow-lists.
