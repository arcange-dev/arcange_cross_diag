# Arcange CrossDiag

Cross-platform PC inventory and guided diagnostics for Windows and Linux.

## Source archive — upload required

The complete application source package is available as **arcange-crossdiag-v1.0.0-rc.2.zip** in the development handoff.

It still needs to be uploaded to this repository at exactly:

`source/arcange-crossdiag-v1.0.0-rc.2.zip`

To add it:
1. Download the ZIP from the project handoff.
2. Open this GitHub repository and choose **Add file → Upload files**.
3. Upload the ZIP into the `source/` directory (create the directory if GitHub asks).
4. Commit the upload to the `upload/crossdiag-v1.0.0-rc.2` branch or to `main`.

The extractor and workflow are already committed on the upload branch. The GitHub integration used for this handoff could commit text files but did not provide a binary-archive upload operation, so the ZIP itself is the one manual upload step.

After uploading, extract it locally with:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/extract-source.ps1
```

The application project will be extracted into `./app`.

## Automatic Windows setup (.exe) build

The workflow is:

`.github/workflows/build-windows-installer.yml`

It runs on pushes, pull requests and manual dispatch. Its build job remains skipped until the source ZIP exists at the exact path above. Once present, it:
1. Extracts the application source.
2. Installs Node.js 22 and stable Rust.
3. Installs npm dependencies.
4. Runs source, security, release and regression checks.
5. Type-checks/builds the React frontend.
6. Checks Rust formatting and runs Rust tests.
7. Builds Tauri Windows bundles.
8. Requires an NSIS setup `.exe` and uploads it as `arcange-crossdiag-windows-setup`.

### Download the generated setup

1. Open the repository's **Actions** tab.
2. Select **Build Windows installer (.exe)**.
3. Open a successful workflow run.
4. Download **arcange-crossdiag-windows-setup** under Artifacts.
5. Extract the artifact ZIP and run the generated CrossDiag setup `.exe`.

The workflow generates an installer artifact; it does not install software on your PC automatically.

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

## Licensing

The repository retains its existing Apache-2.0 license. The source archive includes its own MIT license and third-party notices; review the included notices before redistributing combined packages.
