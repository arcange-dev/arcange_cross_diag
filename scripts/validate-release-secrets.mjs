import process from "node:process";

const publicKey = (process.env.TAURI_UPDATER_PUBLIC_KEY ?? "").trim();
const privateKey = (process.env.TAURI_SIGNING_PRIVATE_KEY ?? "").trim();
const repository = (process.env.GITHUB_REPOSITORY ?? "").trim();

const failures = [];
if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
  failures.push("GITHUB_REPOSITORY must be supplied in owner/repository format.");
}
if (!publicKey) failures.push("TAURI_UPDATER_PUBLIC_KEY repository variable is missing.");
if (!privateKey) failures.push("TAURI_SIGNING_PRIVATE_KEY repository secret is missing.");
if (/PRIVATE KEY|BEGIN .*PRIVATE/i.test(publicKey)) {
  failures.push("The public-key variable appears to contain private-key material.");
}
if (publicKey && privateKey && publicKey === privateKey) {
  failures.push("The public and private signing key values must not be identical.");
}

if (failures.length) {
  process.stderr.write(`${failures.map((item) => `- ${item}`).join("\n")}\n`);
  process.exit(1);
}

process.stdout.write("[OK] Repository identity and updater signing values are present. Secret values were not printed.\n");
process.stdout.write("[INFO] Cryptographic signature verification is performed by Tauri during update installation.\n");
