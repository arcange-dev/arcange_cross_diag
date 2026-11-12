import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const script = await readFile(new URL("../scripts/first-pc-test.ps1", import.meta.url), "utf8");
const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));

test("Windows first-PC test is exposed as an npm script", () => {
  assert.match(packageJson.scripts["test:first-pc:windows"], /first-pc-test\.ps1/);
  assert.match(script, /Get-CommandVersion "npm\.cmd"/);
});

test("first-PC test gathers OS, CPU, RAM and volume facts", () => {
  for (const expected of ["Win32_OperatingSystem", "Win32_ComputerSystem", "Win32_Processor", "Win32_LogicalDisk"]) {
    assert.ok(script.includes(expected), `Expected read-only inventory query ${expected}`);
  }
});

test("first-PC test does not run diagnostic or repair commands", () => {
  for (const forbidden of [/\bsmartctl\s+-[a-z]/i, /\bsfc\s+\/scannow/i, /\bDISM\s+\/Online\s+\/Cleanup-Image\s+\/RestoreHealth/i, /\bwinget\s+upgrade\b/i, /\bFormat-Volume\b/i, /\bClear-Disk\b/i]) {
    assert.doesNotMatch(script, forbidden);
  }
});

test("first-PC test avoids collecting serials, MAC addresses and credentials", () => {
  assert.doesNotMatch(script, /Win32_BIOS|SerialNumber|MacAddress|NetworkAddress|BitLocker|RecoveryKey/i);
});

test("first-PC test writes an explicit local report and states its limitations", () => {
  assert.match(script, /first-pc-test-/);
  assert.match(script, /did not run SMART scans, stress tests, repairs, updates, driver changes, firmware operations or disk writes/);
});
