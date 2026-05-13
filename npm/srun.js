#!/usr/bin/env node
const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { join, resolve } = require("node:path");

const root = resolve(__dirname, "..");
const binaryName = process.platform === "win32" ? "srun.exe" : "srun";
const platformKey = `${process.platform}-${process.arch}`;
const packagedBinary = join(__dirname, "bin", platformKey, binaryName);
const localDevBinary = join(root, "target", "release", binaryName);

function resolveBinary() {
  if (existsSync(packagedBinary)) {
    return packagedBinary;
  }

  if (existsSync(localDevBinary)) {
    return localDevBinary;
  }

  return undefined;
}

const binary = resolveBinary();

if (!binary) {
  console.error(`srun: no bundled binary found for ${platformKey}.`);
  console.error("srun: supported npm platforms are win32-x64, linux-x64, and darwin-arm64.");
  console.error("srun: if you are developing locally, run `cargo build --release` first.");
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), {
  cwd: process.cwd(),
  stdio: "inherit",
});

if (result.error) {
  console.error(`srun: failed to execute binary: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
