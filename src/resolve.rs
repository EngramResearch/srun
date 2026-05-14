use crate::model::{
    CommandSpec, Framework, Intent, ProjectInfo, ResolveFailure, ResolvedCommand, SrunError,
};

pub fn resolve_intent(project: &ProjectInfo, intent: Intent) -> Result<ResolvedCommand, SrunError> {
    if project.is_cargo_only() {
        return resolve_cargo(intent);
    }

    if !project.has_package_json && !project.has_cargo_toml {
        return Err(resolve_error(intent, ResolveFailure::NoProjectDetected));
    }

    let package_manager = project
        .package_manager
        .ok_or_else(|| resolve_error(intent, ResolveFailure::NoPackageManager))?;
    let scripts = project.scripts();
    let candidates = candidates_for(project, intent);

    for candidate in candidates {
        if scripts.contains_key(candidate) {
            return Ok(ResolvedCommand {
                intent,
                command: package_manager.script_command(candidate),
                reason: reason_for(project, intent, candidate),
                script: Some(candidate.to_string()),
            });
        }
    }

    Err(resolve_error(
        intent,
        ResolveFailure::NoMatchingScript {
            candidates: fallback_candidates(&scripts),
        },
    ))
}

pub fn candidates_for(project: &ProjectInfo, intent: Intent) -> Vec<&'static str> {
    match intent {
        Intent::Dev if project.has_framework(Framework::Electron) => vec![
            "dev:electron",
            "electron:dev",
            "desktop:dev",
            "dev",
            "dev:web",
            "tauri:dev",
            "app:dev",
            "start",
            "serve",
            "watch",
        ],
        Intent::Dev if project.has_framework(Framework::Tauri) => vec![
            "tauri:dev",
            "dev:tauri",
            "desktop:dev",
            "dev",
            "dev:web",
            "dev:electron",
            "electron:dev",
            "app:dev",
            "start",
            "serve",
            "watch",
        ],
        Intent::Dev => vec![
            "dev",
            "dev:web",
            "dev:electron",
            "electron:dev",
            "tauri:dev",
            "desktop:dev",
            "app:dev",
            "start",
            "serve",
            "watch",
        ],
        Intent::Build if project.has_framework(Framework::Electron) => vec![
            "build:electron",
            "electron:build",
            "desktop:build",
            "build",
            "build:web",
            "tauri:build",
            "dist",
            "compile",
            "bundle",
        ],
        Intent::Build if project.has_framework(Framework::Tauri) => vec![
            "tauri:build",
            "build:tauri",
            "desktop:build",
            "build",
            "build:web",
            "build:electron",
            "electron:build",
            "dist",
            "compile",
            "bundle",
        ],
        Intent::Build => vec![
            "build",
            "build:web",
            "build:electron",
            "electron:build",
            "tauri:build",
            "dist",
            "compile",
            "bundle",
        ],
        Intent::Installer => vec![
            "installer",
            "installer:win",
            "package",
            "make",
            "dist",
            "release",
            "bundle",
        ],
        Intent::Test => vec!["test", "tests", "test:unit", "test:ci", "ci:test"],
        Intent::Check => vec![
            "check",
            "typecheck",
            "type-check",
            "type:check",
            "tsc",
            "validate",
            "verify",
        ],
        Intent::Preview => vec!["preview", "serve", "start:prod", "preview:web"],
        Intent::Clean => vec!["clean", "clean:all", "reset", "clear"],
        Intent::Setup => vec!["setup", "bootstrap", "install", "deps", "prepare"],
        Intent::Lint => vec!["lint", "lint:check", "eslint", "clippy"],
        Intent::Format => vec!["format", "fmt", "prettier", "format:check"],
    }
}

fn resolve_cargo(intent: Intent) -> Result<ResolvedCommand, SrunError> {
    let command = match intent {
        Intent::Dev => CommandSpec::new("cargo", ["run"]),
        Intent::Build => CommandSpec::new("cargo", ["build"]),
        Intent::Test => CommandSpec::new("cargo", ["test"]),
        Intent::Check => CommandSpec::new("cargo", ["check"]),
        Intent::Lint => CommandSpec::new("cargo", ["clippy"]),
        Intent::Format => CommandSpec::new("cargo", ["fmt"]),
        Intent::Clean => CommandSpec::new("cargo", ["clean"]),
        Intent::Installer | Intent::Preview | Intent::Setup => {
            return Err(resolve_error(intent, ResolveFailure::UnsupportedIntent));
        }
    };

    Ok(ResolvedCommand {
        intent,
        command,
        reason: "Cargo-only project fallback".to_string(),
        script: None,
    })
}

fn reason_for(project: &ProjectInfo, intent: Intent, script: &str) -> String {
    if intent == Intent::Dev
        && project.has_framework(Framework::Electron)
        && script.contains("electron")
    {
        return "Electron project: desktop dev script has priority".to_string();
    }
    if intent == Intent::Build
        && project.has_framework(Framework::Electron)
        && script.contains("electron")
    {
        return "Electron project: desktop build script has priority".to_string();
    }
    if project.has_framework(Framework::Tauri) && script.contains("tauri") {
        return "Tauri project: tauri script has priority".to_string();
    }

    format!("selected first matching {} script", intent.label())
}

fn fallback_candidates(scripts: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    scripts.keys().take(8).cloned().collect()
}

fn resolve_error(intent: Intent, failure: ResolveFailure) -> SrunError {
    SrunError::Resolve { intent, failure }
}

#[cfg(test)]
mod tests {
    use super::{candidates_for, resolve_intent};
    use crate::detect::detect_project;
    use crate::model::{Framework, Intent};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn preserves_base_dev_order_for_simple_projects() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"dev":"next dev"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        let candidates = candidates_for(&project, Intent::Dev);

        assert_eq!(candidates[0], "dev");
    }

    #[test]
    fn resolves_next_dev() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("lock");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"dev":"next dev"},"dependencies":{"next":"latest"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        let resolved = resolve_intent(&project, Intent::Dev).expect("resolved");

        assert_eq!(resolved.command.display(), "pnpm run dev");
    }

    #[test]
    fn prioritizes_electron_dev_script() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("lock");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"dev":"vite","dev:electron":"electron-vite dev"},"devDependencies":{"electron":"latest"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        assert!(project.frameworks.contains(&Framework::Electron));
        let resolved = resolve_intent(&project, Intent::Dev).expect("resolved");

        assert_eq!(resolved.command.display(), "pnpm run dev:electron");
    }

    #[test]
    fn prioritizes_tauri_dev_script() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("lock");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"dev":"vite","tauri:dev":"tauri dev"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        let resolved = resolve_intent(&project, Intent::Dev).expect("resolved");

        assert_eq!(resolved.command.display(), "pnpm run tauri:dev");
    }

    #[test]
    fn resolves_cargo_project() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='x'").expect("cargo");

        let project = detect_project(dir.path()).expect("project");
        let resolved = resolve_intent(&project, Intent::Dev).expect("resolved");

        assert_eq!(resolved.command.display(), "cargo run");
    }

    #[test]
    fn resolves_check_script() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("lock");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"check":"tsc --noEmit"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        let resolved = resolve_intent(&project, Intent::Check).expect("resolved");

        assert_eq!(resolved.command.display(), "pnpm run check");
    }

    #[test]
    fn resolves_cargo_check_and_clean() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]
name='x'",
        )
        .expect("cargo");

        let project = detect_project(dir.path()).expect("project");
        let check = resolve_intent(&project, Intent::Check).expect("check");
        let clean = resolve_intent(&project, Intent::Clean).expect("clean");

        assert_eq!(check.command.display(), "cargo check");
        assert_eq!(clean.command.display(), "cargo clean");
    }

    #[test]
    fn resolves_preview_clean_and_setup_scripts() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"preview":"vite preview","clean":"rimraf dist","setup":"pnpm install"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");

        assert_eq!(
            resolve_intent(&project, Intent::Preview)
                .expect("preview")
                .command
                .display(),
            "npm run preview"
        );
        assert_eq!(
            resolve_intent(&project, Intent::Clean)
                .expect("clean")
                .command
                .display(),
            "npm run clean"
        );
        assert_eq!(
            resolve_intent(&project, Intent::Setup)
                .expect("setup")
                .command
                .display(),
            "npm run setup"
        );
    }

    #[test]
    fn custom_script_returns_candidates_instead_of_guessing() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{"scripts":{"banana":"vite"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");
        let error = resolve_intent(&project, Intent::Dev).expect_err("should not guess");

        assert!(error.to_string().contains("banana"));
    }
}
