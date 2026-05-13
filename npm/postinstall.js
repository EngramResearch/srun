const { spawnSync } = require("node:child_process");
const { resolve } = require("node:path");

const root = resolve(__dirname, "..");

const result = spawnSync("cargo", ["build", "--release"], {
  cwd: root,
  stdio: "inherit",
  shell: process.platform === "win32",
});

if (result.error) {
  console.error("srun: cargo is required to build the npm package binary.");
  console.error(`srun: ${result.error.message}`);
  process.exit(1);
}

process.exit(result.status ?? 1);
