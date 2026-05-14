<p align="center">
  <img src="https://raw.githubusercontent.com/EngramResearch/srun/main/srun-logo.png" alt="srun logo" width="620">
</p>

<h1 align="center">srun</h1>

<p align="center">
  <strong>Universal Smart Project Runner.</strong>
</p>

`srun` translates a developer intent into the concrete command for the current project.

```bash
srun dev
srun build
srun installer
srun test
srun check
srun preview
srun clean
srun setup
srun lint
srun format
srun info
srun list
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

The npm package includes prebuilt binaries for supported platforms. Rust/Cargo is only required when building from source.

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

## Commands

Core commands:

- `srun dev` — run the project in development mode.
- `srun build` — build the project.
- `srun installer` — create installer/package/release artifacts when a matching script exists.
- `srun test` — run tests.
- `srun check` — run validation/typecheck. Alias: `srun typecheck`.
- `srun preview` — run preview/serve production build when available.
- `srun clean` — clean build output/cache.
- `srun setup` — run setup/bootstrap/dependency scripts. Aliases: `srun install`, `srun bootstrap`.
- `srun lint` — run linting.
- `srun format` — run formatting.
- `srun info` — show detected project type, package manager, warnings, and resolved commands.
- `srun list` — show resolved intents plus all package scripts.

Safe flags for executable commands:

- `--dry-run` prints the resolved command without executing it.
- `--verbose` prints detection and resolution details.

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

srun check --dry-run
# cargo check

srun clean --dry-run
# cargo clean
```

TypeScript validation:

```json
{
  "scripts": {
    "check": "tsc --noEmit"
  }
}
```

```bash
srun check --dry-run
# pnpm run check
```

Discovery:

```bash
srun list
```

Shows resolved intents and all package scripts.

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

1. Update the version in `package.json` and `Cargo.toml`.
2. Run:

```bash
npm run release:check
cargo fmt -- --check
cargo check
cargo test
```

3. Commit, push, and create a tag:

```bash
git tag v0.1.5
git push origin main v0.1.5
```

The workflow builds platform binaries, packages them into the single root npm package, publishes `@engramresearch/srun` to npmjs, then publishes the same package as a GitHub Packages mirror.

Required GitHub secret:

```text
NPM_TOKEN
```

Use an npm automation/granular token that can publish under `@engramresearch` and bypass 2FA for CI.

GitHub Packages uses the workflow `GITHUB_TOKEN`; no extra secret is required.

## Current limitations

- Prebuilt npm binaries currently target Windows x64, Linux x64, and macOS arm64.
- Monorepo scopes such as `srun dev web` are detected as a future extension but not fully implemented yet.
- Interactive fallback for custom scripts is not implemented; `srun` reports candidates instead of guessing.
- Colors and shell integration are intentionally omitted from the MVP.
