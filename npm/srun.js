#!/usr/bin/env node
const { spawnSync } = require("node:child_process");
const { existsSync } = require("node:fs");
const { dirname, join, resolve } = require("node:path");

const root = resolve(__dirname, "..");
const binaryName = process.platform === "win32" ? "srun.exe" : "srun";

const platformPackages = {
  "darwin-arm64": "@engramresearch/srun-darwin-arm64",
  "darwin-x64": "@engramresearch/srun-darwin-x64",
  "linux-x64": "@engramresearch/srun-linux-x64",
  "win32-x64": "@engramresearch/srun-win32-x64",
};

function platformBinary() {
  const packageName = platformPackages[`${process.platform}-${process.arch}`];
  if (!packageName) {
    return undefined;
  }

  try {
    const packageJson = require.resolve(`${packageName}/package.json`);
    return join(dirname(packageJson), "bin", binaryName);
  } catch {
    return undefined;
  }
}

function localDevBinary() {
  const binary = join(root, "target", "release", binaryName);
  if (existsSync(binary)) {
    return binary;
  }

  if (!existsSync(join(root, "Cargo.toml"))) {
    return undefined;
  }

  const result = spawnSync("cargo", ["build", "--release"], {
    cwd: root,
    stdio: "inherit",
    shell: process.platform === "win32",
  });

  if (result.error) {
    console.error(`srun: failed to build local Rust binary: ${result.error.message}`);
    process.exit(1);
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }

  return existsSync(binary) ? binary : undefined;
}

const binary = platformBinary() ?? localDevBinary();

if (!binary || !existsSync(binary)) {
  console.error(`srun: no binary package found for ${process.platform}-${process.arch}.`);
  console.error("srun: install from a supported platform package or build from source with Cargo.");
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
