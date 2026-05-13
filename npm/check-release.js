const { readFileSync } = require("node:fs");

const cargoToml = readFileSync("Cargo.toml", "utf8");
const packageJson = JSON.parse(readFileSync("package.json", "utf8"));
const cargoVersion = cargoToml.match(/^version = "([^"]+)"/m)?.[1];

if (!cargoVersion) {
  throw new Error("Cargo.toml version not found");
}

if (packageJson.version !== cargoVersion) {
  throw new Error(`version mismatch: package.json=${packageJson.version}, Cargo.toml=${cargoVersion}`);
}

if (packageJson.optionalDependencies) {
  throw new Error("single-package release must not use optionalDependencies");
}

console.log(`release metadata OK for ${packageJson.version}`);
