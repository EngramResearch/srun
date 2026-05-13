const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const rootPackage = JSON.parse(readFileSync("package.json", "utf8"));
const platforms = [
  "darwin-arm64",
  "darwin-x64",
  "linux-x64",
  "win32-x64",
];

for (const platform of platforms) {
  const packageName = `@engramresearch/srun-${platform}`;
  const packageJsonPath = join("npm", "platforms", platform, "package.json");
  const platformPackage = JSON.parse(readFileSync(packageJsonPath, "utf8"));

  if (platformPackage.name !== packageName) {
    throw new Error(`${packageJsonPath}: expected name ${packageName}, got ${platformPackage.name}`);
  }

  if (platformPackage.version !== rootPackage.version) {
    throw new Error(
      `${packageJsonPath}: expected version ${rootPackage.version}, got ${platformPackage.version}`,
    );
  }

  if (rootPackage.optionalDependencies[packageName] !== rootPackage.version) {
    throw new Error(`package.json: optional dependency ${packageName} must be ${rootPackage.version}`);
  }
}

console.log(`release package metadata OK for ${rootPackage.version}`);
