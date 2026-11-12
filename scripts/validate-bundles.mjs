import { readdir } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const bundleRoot = path.resolve("src-tauri/target/release/bundle");
const platform = process.platform;

async function walk(directory) {
  let entries;
  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch {
    return [];
  }
  const files = [];
  for (const entry of entries) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await walk(fullPath));
    else files.push(fullPath);
  }
  return files;
}

const files = await walk(bundleRoot);
const names = files.map((file) => path.basename(file));
let acceptable = [];
if (platform === "win32") {
  acceptable = files.filter((file) => /\.(?:exe|msi)$/i.test(file));
} else if (platform === "linux") {
  acceptable = files.filter((file) => /\.(?:AppImage|deb|rpm)$/i.test(file));
} else {
  process.stderr.write(`Bundle validation is configured for Windows and Linux, not ${platform}.\n`);
  process.exit(2);
}

if (acceptable.length === 0) {
  process.stderr.write(`No expected ${platform} installer was found under ${bundleRoot}.\n`);
  process.stderr.write(`Files found: ${names.length ? names.join(", ") : "none"}\n`);
  process.exit(1);
}

process.stdout.write(`[OK] Found ${acceptable.length} ${platform} installer bundle(s):\n`);
for (const file of acceptable) process.stdout.write(`- ${path.relative(process.cwd(), file)}\n`);
