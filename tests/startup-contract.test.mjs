import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const read = (path) => readFile(new URL(`../${path}`, import.meta.url), "utf8");

const [mainRs, libRs, tauriConfigText, viteConfig, capabilityText, indexHtml, mainTsx, packageJsonText, installerWorkflow, releaseWorkflow] =
  await Promise.all([
    read("src-tauri/src/main.rs"),
    read("src-tauri/src/lib.rs"),
    read("src-tauri/tauri.conf.json"),
    read("vite.config.ts"),
    read("src-tauri/capabilities/default.json"),
    read("index.html"),
    read("src/main.tsx"),
    read("package.json"),
    read(".github/workflows/build-windows-installer.yml"),
    read(".github/workflows/release.yml"),
  ]);

const tauriConfig = JSON.parse(tauriConfigText);
const capability = JSON.parse(capabilityText);
const packageJson = JSON.parse(packageJsonText);

test("release builds hide the console, so startup failures must be written to a file", () => {
  assert.match(mainRs, /windows_subsystem = "windows"/, "the Windows subsystem attribute must stay in place");
  assert.match(mainRs, /arcange_crossdiag_lib::run\(\)/, "main must call the library entry point");
  assert.match(libRs, /fn report_startup_failure\(/, "a startup failure reporter must exist");
  assert.match(libRs, /if let Err\(error\) = startup_result/, "run() must handle the startup result instead of panicking");
  assert.match(libRs, /report_startup_failure\(&error\.to_string\(\)\)/, "run() must report the failure it receives");
  assert.match(libRs, /startup-error\.log/, "the failure report must name a log file the user can send");
  assert.doesNotMatch(libRs, /\.expect\("error while running Arcange CrossDiag"\)/, "a silent panic must not replace the report");
  assert.match(libRs, /std::io::Write::write_all/, "the reporter must call write_all through the trait explicitly");
  assert.doesNotMatch(libRs.slice(0, 900), /io::\{Read, Write\}/, "a module-level Write import makes File::by_ref() ambiguous (E0034)");
});

test("the development server address is identical in Vite and Tauri", () => {
  const port = viteConfig.match(/port:\s*(\d+)/)?.[1];
  const host = viteConfig.match(/host:\s*"([^"]+)"/)?.[1];
  assert.ok(port, "vite.config.ts must declare a fixed port");
  assert.ok(host, "vite.config.ts must declare a fixed host");
  assert.match(viteConfig, /strictPort:\s*true/, "the development port must not silently change");
  assert.equal(tauriConfig.build.devUrl, `http://${host}:${port}`, "tauri build.devUrl must match the Vite server");
  assert.equal(tauriConfig.build.beforeDevCommand, "npm run dev", "Tauri must start the Vite server through npm");
  assert.equal(tauriConfig.build.frontendDist, "../dist", "the frontend must build into dist for bundling");
  assert.match(packageJson.scripts.dev, new RegExp(`--port ${port}`), "npm run dev must serve the port Tauri expects");
  assert.equal(packageJson.scripts.start, "tauri dev", "npm run start must launch the native desktop application");
});

test("the capability applies to the real window label", () => {
  const windowLabel = tauriConfig.app.windows[0].label ?? "main";
  assert.ok(Array.isArray(capability.windows), "the capability must declare windows");
  assert.ok(capability.windows.includes(windowLabel), `capability windows must include the window label '${windowLabel}'`);
  assert.ok(capability.permissions.includes("core:default"), "core permissions must be granted to the main window");
  assert.ok(capability.permissions.includes("updater:default"), "the updater permission the UI uses must be granted");
});

test("the frontend mount point exists in both the HTML shell and the React entry point", () => {
  assert.match(indexHtml, /<div id="root"><\/div>/, "index.html must provide the root element");
  assert.match(indexHtml, /src="\/src\/main\.tsx"/, "index.html must load the React entry point");
  assert.match(mainTsx, /createRoot\(document\.getElementById\("root"\)!\)/, "main.tsx must mount on the root element");
  assert.match(mainTsx, /import App from "\.\/App"/, "main.tsx must render the application shell");
});

test("the installers are produced with a predictable name and verified before upload", () => {
  assert.match(installerWorkflow, /Arcange-CrossDiag-Setup-x64\.exe/, "the Windows setup must be renamed to the documented installer name");
  assert.match(installerWorkflow, /-bundles nsis/, "prerelease builds must request the NSIS bundle");
  assert.match(installerWorkflow, /\.Length -lt 1MB/, "an undersized installer must fail the job");
  assert.match(installerWorkflow, /\.Length -le 0/, "an empty installer must fail the job");
  assert.match(installerWorkflow, /if-no-files-found: error/, "a missing installer must fail the job");
  assert.match(installerWorkflow, /npm run test:source/, "the installer job must keep the source audit");
  assert.match(installerWorkflow, /npm run test:rust/, "the installer job must keep the Rust tests");
});

test("a release without signing secrets is skipped instead of failing", () => {
  assert.match(releaseWorkflow, /signing-readiness/, "release signing readiness must be checked first");
  assert.match(releaseWorkflow, /needs\.signing-readiness\.outputs\.signed == 'true'/, "publishing must depend on signing readiness");
  assert.match(releaseWorkflow, /uploadUpdaterJson: true/, "signed releases must still publish the updater manifest");
});
