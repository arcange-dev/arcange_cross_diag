# Windows launch troubleshooting

## Why "I ran the app and nothing opened" is usually correct behaviour

CrossDiag has two different run modes, and only one of them opens a desktop window:

| Command | What actually starts | Desktop window |
| --- | --- | --- |
| `npm run dev` | the Vite web server on `http://127.0.0.1:1420` | **no** |
| `npm run start` (or `npm run tauri dev`) | the native Tauri application, which loads the Vite server | yes |
| the installed `Arcange-CrossDiag-Setup-x64.exe` | the native application using the bundled frontend | yes |

`npm run dev` is the *frontend* development server that Tauri starts for you. On its own it
never creates a Windows process you can see in Task Manager and it never opens a window. If
the only thing you ran was `npm run dev`, nothing is broken: there was simply no desktop
application in that command.

The native application also requires three things that a fresh Windows machine often lacks:

1. the Microsoft **WebView2 runtime** (preinstalled on current Windows 10/11, missing on
   some stripped or older images),
2. the **Rust MSVC toolchain** and the **Visual Studio C++ build tools** for a local build
   (`npm run start` compiles `src-tauri` on first run),
3. **Node.js 22 or newer** and installed frontend packages (`npm install`).

A downloaded installer needs only the WebView2 runtime, because the executable is prebuilt.

## Release builds fail silently by design, so they now write a log

Release executables are compiled with `windows_subsystem = "windows"`, which means Windows
gives them no console. Before v1.0.0-rc.3 a startup failure therefore produced exactly the
symptom people report: no window, no console text, no Task Manager entry after the process
exits, and no evidence anywhere.

`run()` now records every startup failure to:

```
%LOCALAPPDATA%\Arcange CrossDiag\startup-error.log
```

The file contains the Unix timestamp, the application version, the platform and the exact
error. Send that file with any bug report.

## Collect evidence in one step

Read-only collector (writes a report, changes nothing):

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/windows-launch-diagnosis.ps1
```

It records the OS build and architecture, Node.js, npm, rustc, cargo, rustup targets, MSVC
build tools, the WebView2 runtime version, the Visual C++ runtime DLLs, the project state
(dependencies installed, Tauri CLI present, `devUrl`, window label), recent Windows
Application error events, and any recorded startup-error log.

To also start the native application, check whether a window appears, and capture its output:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/windows-launch-diagnosis.ps1 -Launch
```

`-Launch` prints the launcher exit code, whether an `Arcange CrossDiag` process existed, and
whether it owned a visible window. It stops only the processes it started itself.

Reports are written to `reports\` and are safe to share: they contain versions, paths and
error text, no serial numbers, credentials or user documents.

## Decision table

| Evidence in the report | Situation | Fix |
| --- | --- | --- |
| `node_modules MISSING`, `Tauri CLI not installed` | frontend never installed | `npm install` |
| `rustc`/`cargo` NOT FOUND | no native toolchain | install Rust (MSVC toolchain) and the C++ build tools |
| `MSVC C++ build tools NOT installed` | Rust cannot link | install Visual Studio Build Tools with "Desktop development with C++" |
| `WebView2 runtime NOT DETECTED` | window cannot render | install the Microsoft Edge WebView2 Evergreen runtime |
| launcher exit code `0`, no process, no output | you ran `npm run dev` | use `npm run start` |
| native process RUNNING but `Window visible: False` | window created off-screen or still loading | attach `%LOCALAPPDATA%\Arcange CrossDiag\startup-error.log` |
| process exits immediately | Rust or asset startup failure | read `startup-error.log` |
| Vite reachable in a browser but no window | Tauri not launched | run `npm run start` |

## What this repository does *not* claim

The collector cannot inspect a machine it does not run on. Every statement about your PC in a
bug report must come from that report, not from an assumption.
