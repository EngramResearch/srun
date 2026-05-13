use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Intent {
    Dev,
    Build,
    Installer,
    Test,
    Lint,
    Format,
}

impl Intent {
    pub const EXECUTABLE: [Intent; 6] = [
        Intent::Dev,
        Intent::Build,
        Intent::Installer,
        Intent::Test,
        Intent::Lint,
        Intent::Format,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Intent::Dev => "dev",
            Intent::Build => "build",
            Intent::Installer => "installer",
            Intent::Test => "test",
            Intent::Lint => "lint",
            Intent::Format => "format",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PackageManager {
    Pnpm,
    Bun,
    Yarn,
    Npm,
}

impl PackageManager {
    pub fn label(self) -> &'static str {
        match self {
            PackageManager::Pnpm => "pnpm",
            PackageManager::Bun => "bun",
            PackageManager::Yarn => "yarn",
            PackageManager::Npm => "npm",
        }
    }

    pub fn script_command(self, script: &str) -> CommandSpec {
        match self {
            PackageManager::Pnpm => CommandSpec::new("pnpm", ["run", script]),
            PackageManager::Bun => CommandSpec::new("bun", ["run", script]),
            PackageManager::Yarn => CommandSpec::new("yarn", [script]),
            PackageManager::Npm => CommandSpec::new("npm", ["run", script]),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Framework {
    Electron,
    Tauri,
    Next,
    Vite,
    Turbo,
    Nx,
}

impl Framework {
    pub fn label(self) -> &'static str {
        match self {
            Framework::Electron => "Electron",
            Framework::Tauri => "Tauri",
            Framework::Next => "Next.js",
            Framework::Vite => "Vite",
            Framework::Turbo => "TurboRepo",
            Framework::Nx => "NX",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackageJson {
    pub scripts: BTreeMap<String, String>,
    pub dependencies: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub root: PathBuf,
    pub package_manager: Option<PackageManager>,
    pub package_json: Option<PackageJson>,
    pub frameworks: BTreeSet<Framework>,
    pub has_cargo_toml: bool,
    pub has_package_json: bool,
    pub is_monorepo: bool,
    pub warnings: Vec<String>,
    pub traces: Vec<String>,
}

impl ProjectInfo {
    pub fn project_type(&self) -> String {
        if self.is_cargo_only() {
            return "Cargo".to_string();
        }

        let mut labels: Vec<&str> = self
            .frameworks
            .iter()
            .map(|framework| framework.label())
            .collect();
        if self.is_monorepo {
            labels.push("Monorepo");
        }

        if labels.is_empty() {
            if self.has_package_json {
                "Node.js".to_string()
            } else {
                "Unknown".to_string()
            }
        } else {
            labels.join(" + ")
        }
    }

    pub fn scripts(&self) -> BTreeMap<String, String> {
        self.package_json
            .as_ref()
            .map(|package_json| package_json.scripts.clone())
            .unwrap_or_default()
    }

    pub fn has_framework(&self, framework: Framework) -> bool {
        self.frameworks.contains(&framework)
    }

    pub fn is_cargo_only(&self) -> bool {
        self.has_cargo_toml && !self.has_package_json
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    pub fn new<I, S>(program: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    pub fn display(&self) -> String {
        if self.args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, self.args.join(" "))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCommand {
    pub intent: Intent,
    pub command: CommandSpec,
    pub reason: String,
    pub script: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveFailure {
    NoProjectDetected,
    NoPackageManager,
    NoMatchingScript { candidates: Vec<String> },
    UnsupportedIntent,
}

#[derive(Debug)]
pub enum SrunError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Resolve {
        intent: Intent,
        failure: ResolveFailure,
    },
    ProcessSpawn {
        command: String,
        source: std::io::Error,
    },
}

impl fmt::Display for SrunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SrunError::Io { path, source } => {
                write!(formatter, "failed to read {}: {}", path.display(), source)
            }
            SrunError::Json { path, source } => {
                write!(formatter, "failed to parse {}: {}", path.display(), source)
            }
            SrunError::Resolve { intent, failure } => write!(
                formatter,
                "could not resolve {}: {}",
                intent.label(),
                failure
            ),
            SrunError::ProcessSpawn { command, source } => {
                write!(formatter, "failed to execute `{}`: {}", command, source)
            }
        }
    }
}

impl std::error::Error for SrunError {}

impl fmt::Display for ResolveFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveFailure::NoProjectDetected => {
                write!(formatter, "no supported project files found")
            }
            ResolveFailure::NoPackageManager => write!(
                formatter,
                "package.json exists but no package manager could be selected"
            ),
            ResolveFailure::NoMatchingScript { candidates } => {
                if candidates.is_empty() {
                    write!(formatter, "no matching script found")
                } else {
                    write!(
                        formatter,
                        "no matching script found. Possible candidates: {}",
                        candidates.join(", ")
                    )
                }
            }
            ResolveFailure::UnsupportedIntent => {
                write!(formatter, "intent is not supported for this project type")
            }
        }
    }
}
