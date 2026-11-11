# Arcange CrossDiag

Cross-platform PC inventory and guided diagnostics for Windows and Linux.

## Source archive

The complete application source for **v1.0.0-rc.2** is stored in:

`source/arcange-crossdiag-v1.0.0-rc.2.zip`

Extract it with the helper script:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/extract-source.ps1
```

The project will be extracted into `./app`.

## Build a Windows setup executable

This repository includes an automatic GitHub Actions workflow:

`.github/workflows/build-windows-installer.yml`

It runs on pushes, pull requests, and manual dispatch. The workflow:
1. Extracts the full application source.
2. Installs Node.js 22 and stable Rust.
3. Installs npm dependencies.
4. Runs source, security, release and regression checks.
5. Type-checks/builds the React frontend.
6. Checks Rust formatting and runs Rust tests.
7. Builds Tauri Windows bundles.
8. Requires an NSIS installer `.exe` and uploads it as the `arcange-crossdiag-windows-setup` artifact.

### Download the generated setup

1. Open the repository's **Actions** tab.
2. Select **Build Windows installer (.exe)**.
3. Open a successful workflow run.
4. Download **arcange-crossdiag-windows-setup** under Artifacts.
5. Extract the artifact ZIP and run the generated CrossDiag setup `.exe`.

The workflow produces a CI artifact; it does not silently install the application on your PC. Review and test the installer before distributing it.

## Build locally

Prerequisites: Node.js 22+, npm 10+, stable Rust/Cargo, Microsoft C++ Build Tools and WebView2 Runtime. See the [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/).

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/extract-source.ps1
Set-Location app
npm install
npm run doctor
npm run tauri dev
npm run build:desktop
```

## License

The repository's automation/wrapper files retain the repository's existing Apache-2.0 license. The bundled application source archive contains its own MIT license and third-party notices. Review the included notices before redistributing combined packages.
