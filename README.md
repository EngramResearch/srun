# srun

`srun` is a Universal Smart Project Runner. It translates a developer intent into the concrete command for the current project.

```bash
srun dev
srun build
srun installer
srun test
srun lint
srun format
srun info
```

The goal is to reduce cognitive load when switching between projects, package managers, frameworks, and custom script names.

## Install locally

With Cargo:

```bash
cargo install --path .
```

With npm from the project directory:

```bash
npm install -g .
```

After publishing:

```bash
npm install -g @engramresearch/srun
```

The npm package installs a prebuilt platform binary when available. Rust/Cargo is only required when building from source.

Or run during development:

```bash
cargo run -- info
cargo run -- dev --dry-run
```

## What it detects

Package managers:

- `pnpm-lock.yaml` -> `pnpm`
- `bun.lockb` or `bun.lock` -> `bun`
- `yarn.lock` -> `yarn`
- `package-lock.json` -> `npm`
- `package.json` without lockfile -> `npm` with warning

If multiple lockfiles exist, priority is:

```text
pnpm > bun > yarn > npm
```

Project markers:

- Electron: `electron`, `electron-builder`, `electron-vite`, `electron/`, `electron.vite.config.*`
- Tauri: `src-tauri/`, `tauri.conf.json`, tauri scripts/dependencies
- Next.js: `next.config.*` or `next`
- Vite: `vite.config.*` or `vite`
- TurboRepo: `turbo.json`
- NX: `nx.json`
- Monorepo: `apps/`, `packages/`, TurboRepo or NX markers
- Cargo-only: `Cargo.toml` without `package.json`

## Resolution examples

Next.js:

```json
{
  "scripts": {
    "dev": "next dev"
  }
}
```

```bash
srun dev --dry-run
# pnpm run dev
```

Electron:

```json
{
  "scripts": {
    "dev": "vite",
    "dev:electron": "electron-vite dev"
  }
}
```

```bash
srun dev --dry-run
# pnpm run dev:electron
```

Tauri:

```json
{
  "scripts": {
    "tauri:dev": "tauri dev"
  }
}
```

```bash
srun dev --dry-run
# pnpm run tauri:dev
```

Cargo-only:

```bash
srun dev --dry-run
# cargo run
```

## Info and verbose mode

```bash
srun info
```

Prints project type, package manager, warnings, and resolved commands.

```bash
srun dev --verbose --dry-run
```

Shows detection and resolution phases before printing the command.

## Release process

Releases are published by GitHub Actions from version tags.

1. Update the version in `package.json`, `Cargo.toml`, and `npm/platforms/*/package.json`.
2. Ensure root `optionalDependencies` point to the same version.
3. Run:

```bash
npm run release:check
cargo fmt -- --check
cargo check
cargo test
```

4. Commit, push, and create a tag:

```bash
git tag v0.1.2
git push origin main v0.1.2
```

The workflow publishes platform binary packages first, then the root npm package.

Required GitHub secret:

```text
NPM_TOKEN
```

Use an npm automation/granular token that can publish under `@engramresearch` and bypass 2FA for CI.

## Current limitations

- Prebuilt npm binaries currently target Windows x64, Linux x64, macOS x64, and macOS arm64.
- Monorepo scopes such as `srun dev web` are detected as a future extension but not fully implemented yet.
- Interactive fallback for custom scripts is not implemented; `srun` reports candidates instead of guessing.
- Colors and shell integration are intentionally omitted from the MVP.
