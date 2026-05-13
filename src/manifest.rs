use crate::model::{PackageJson, SrunError};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
struct RawPackageJson {
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    #[serde(default)]
    dependencies: BTreeMap<String, serde_json::Value>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: BTreeMap<String, serde_json::Value>,
}

pub fn read_package_json(root: &Path) -> Result<Option<PackageJson>, SrunError> {
    let path = root.join("package.json");
    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path).map_err(|source| SrunError::Io {
        path: path.clone(),
        source,
    })?;
    let raw: RawPackageJson = serde_json::from_str(&content).map_err(|source| SrunError::Json {
        path: path.clone(),
        source,
    })?;

    let dependencies = raw
        .dependencies
        .into_keys()
        .chain(raw.dev_dependencies.into_keys())
        .collect::<BTreeSet<_>>();

    Ok(Some(PackageJson {
        scripts: raw.scripts,
        dependencies,
    }))
}

#[cfg(test)]
mod tests {
    use super::read_package_json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn reads_scripts_and_dependencies() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("package.json"),
            r#"{
              "scripts": { "dev": "vite" },
              "dependencies": { "vite": "latest" },
              "devDependencies": { "electron": "latest" }
            }"#,
        )
        .expect("write package");

        let package = read_package_json(dir.path())
            .expect("read package")
            .expect("package");

        assert_eq!(package.scripts.get("dev"), Some(&"vite".to_string()));
        assert!(package.dependencies.contains("vite"));
        assert!(package.dependencies.contains("electron"));
    }

    #[test]
    fn reports_invalid_json() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("package.json"), "{").expect("write package");

        let error = read_package_json(dir.path()).expect_err("invalid json should fail");

        assert!(error.to_string().contains("failed to parse"));
    }
}
