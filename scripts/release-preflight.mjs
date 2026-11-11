import { readFile } from "node:fs/promises";
import process from "node:process";

const pkg = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
const tauri = JSON.parse(await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
const cargo = await readFile(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const explicitTag = process.env.RELEASE_TAG?.trim();
const refTag = process.env.GITHUB_REF?.startsWith("refs/tags/") ? process.env.GITHUB_REF_NAME?.trim() : undefined;
const enforcedTag = explicitTag || refTag;
const tag = enforcedTag ?? `v${pkg.version}`;
const expectedTag = `v${pkg.version}`;

const failures = [];
if (tauri.version !== pkg.version) failures.push(`Tauri version ${tauri.version} does not match package version ${pkg.version}.`);
if (cargoVersion !== pkg.version) failures.push(`Cargo version ${cargoVersion} does not match package version ${pkg.version}.`);
if (enforcedTag && tag !== expectedTag) failures.push(`Release tag ${tag} must exactly match ${expectedTag}.`);
if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(pkg.version)) failures.push(`Version ${pkg.version} is not a valid SemVer value.`);
if (failures.length) {
  process.stderr.write(`${failures.map((item) => `- ${item}`).join("\n")}\n`);
  process.exit(1);
}
process.stdout.write(
  enforcedTag
    ? `[OK] Release tag ${tag} matches package, Tauri and Cargo version ${pkg.version}.\n`
    : `[OK] Package, Tauri and Cargo versions are ${pkg.version}; no release tag in this context, so the tag gate is not applied.\n`,
);
