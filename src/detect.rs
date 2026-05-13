use crate::manifest::read_package_json;
use crate::model::{Framework, PackageManager, ProjectInfo, SrunError};
use std::collections::BTreeSet;
use std::path::Path;

pub fn detect_project(root: &Path) -> Result<ProjectInfo, SrunError> {
    let package_json = read_package_json(root)?;
    let has_package_json = package_json.is_some();
    let has_cargo_toml = root.join("Cargo.toml").exists();
    let (package_manager, mut warnings, mut traces) =
        detect_package_manager(root, has_package_json);
    let mut frameworks = BTreeSet::new();

    if let Some(package_json) = &package_json {
        let deps = &package_json.dependencies;
        let scripts = package_json
            .scripts
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();

        if deps.contains("electron")
            || deps.contains("electron-builder")
            || deps.contains("electron-vite")
            || root.join("electron").is_dir()
            || has_file_prefix(root, "electron.vite.config.")
        {
            frameworks.insert(Framework::Electron);
            traces.push("found electron markers".to_string());
        }

        if root.join("src-tauri").is_dir()
            || root.join("tauri.conf.json").exists()
            || deps.iter().any(|dep| dep.contains("tauri"))
            || scripts.iter().any(|script| script.contains("tauri"))
        {
            frameworks.insert(Framework::Tauri);
            traces.push("found tauri markers".to_string());
        }

        if deps.contains("next") || has_file_prefix(root, "next.config.") {
            frameworks.insert(Framework::Next);
            traces.push("found next markers".to_string());
        }

        if deps.contains("vite") || has_file_prefix(root, "vite.config.") {
            frameworks.insert(Framework::Vite);
            traces.push("found vite markers".to_string());
        }
    } else {
        if root.join("src-tauri").is_dir() || root.join("tauri.conf.json").exists() {
            frameworks.insert(Framework::Tauri);
            traces.push("found tauri markers".to_string());
        }
    }

    if root.join("turbo.json").exists() {
        frameworks.insert(Framework::Turbo);
        traces.push("found turbo.json".to_string());
    }

    if root.join("nx.json").exists() {
        frameworks.insert(Framework::Nx);
        traces.push("found nx.json".to_string());
    }

    let is_monorepo = root.join("apps").is_dir()
        || root.join("packages").is_dir()
        || frameworks.contains(&Framework::Turbo)
        || frameworks.contains(&Framework::Nx);

    if is_monorepo {
        traces.push("found monorepo markers".to_string());
    }

    warnings.retain(|warning| !warning.is_empty());

    Ok(ProjectInfo {
        root: root.to_path_buf(),
        package_manager,
        package_json,
        frameworks,
        has_cargo_toml,
        has_package_json,
        is_monorepo,
        warnings,
        traces,
    })
}

fn detect_package_manager(
    root: &Path,
    has_package_json: bool,
) -> (Option<PackageManager>, Vec<String>, Vec<String>) {
    let mut found = Vec::new();
    let mut traces = Vec::new();

    if root.join("pnpm-lock.yaml").exists() {
        found.push(PackageManager::Pnpm);
        traces.push("found pnpm-lock.yaml".to_string());
    }
    if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
        found.push(PackageManager::Bun);
        traces.push("found bun lockfile".to_string());
    }
    if root.join("yarn.lock").exists() {
        found.push(PackageManager::Yarn);
        traces.push("found yarn.lock".to_string());
    }
    if root.join("package-lock.json").exists() {
        found.push(PackageManager::Npm);
        traces.push("found package-lock.json".to_string());
    }

    let selected = [
        PackageManager::Pnpm,
        PackageManager::Bun,
        PackageManager::Yarn,
        PackageManager::Npm,
    ]
    .into_iter()
    .find(|package_manager| found.contains(package_manager))
    .or_else(|| has_package_json.then_some(PackageManager::Npm));

    let mut warnings = Vec::new();
    if found.len() > 1 {
        if let Some(selected) = selected {
            warnings.push(format!(
                "Multiple package managers detected. Using {}.",
                selected.label()
            ));
        }
    } else if found.is_empty() && has_package_json {
        warnings.push("No lockfile found. Using npm.".to_string());
    }

    (selected, warnings, traces)
}

fn has_file_prefix(root: &Path, prefix: &str) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        entry
            .file_name()
            .to_str()
            .map(|file_name| file_name.starts_with(prefix))
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::detect_project;
    use crate::model::{Framework, PackageManager};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn selects_pnpm_over_other_lockfiles() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("package.json"), "{}").expect("package");
        fs::write(dir.path().join("pnpm-lock.yaml"), "").expect("pnpm");
        fs::write(dir.path().join("package-lock.json"), "").expect("npm");

        let project = detect_project(dir.path()).expect("project");

        assert_eq!(project.package_manager, Some(PackageManager::Pnpm));
        assert!(project
            .warnings
            .iter()
            .any(|warning| warning.contains("Multiple")));
    }

    #[test]
    fn falls_back_to_npm_for_package_json_without_lock() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("package.json"), "{}").expect("package");

        let project = detect_project(dir.path()).expect("project");

        assert_eq!(project.package_manager, Some(PackageManager::Npm));
        assert!(project
            .warnings
            .iter()
            .any(|warning| warning.contains("No lockfile")));
    }

    #[test]
    fn detects_electron_and_vite() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"electron":"latest","vite":"latest"}}"#,
        )
        .expect("package");

        let project = detect_project(dir.path()).expect("project");

        assert!(project.frameworks.contains(&Framework::Electron));
        assert!(project.frameworks.contains(&Framework::Vite));
    }

    #[test]
    fn detects_tauri_next_and_monorepo() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{"dependencies":{"next":"latest"},"scripts":{"tauri:dev":"tauri dev"}}"#,
        )
        .expect("package");
        fs::create_dir(dir.path().join("apps")).expect("apps");

        let project = detect_project(dir.path()).expect("project");

        assert!(project.frameworks.contains(&Framework::Tauri));
        assert!(project.frameworks.contains(&Framework::Next));
        assert!(project.is_monorepo);
    }

    #[test]
    fn detects_cargo_only() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname='x'").expect("cargo");

        let project = detect_project(dir.path()).expect("project");

        assert!(project.is_cargo_only());
    }
}
