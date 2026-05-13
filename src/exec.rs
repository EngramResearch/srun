use crate::model::{CommandSpec, SrunError};
use std::io;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

pub fn run_command(command: &CommandSpec, cwd: &Path) -> Result<ExitStatus, SrunError> {
    spawn_command(command, cwd).map_err(|source| SrunError::ProcessSpawn {
        command: command.display(),
        source,
    })
}

fn spawn_command(command: &CommandSpec, cwd: &Path) -> io::Result<ExitStatus> {
    match spawn_program(&command.program, &command.args, cwd) {
        Ok(status) => Ok(status),
        Err(error) if should_try_windows_cmd_shim(&command.program, &error) => {
            spawn_program(&format!("{}.cmd", command.program), &command.args, cwd)
        }
        Err(error) => Err(error),
    }
}

fn spawn_program(program: &str, args: &[String], cwd: &Path) -> io::Result<ExitStatus> {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
}

fn should_try_windows_cmd_shim(program: &str, error: &io::Error) -> bool {
    cfg!(windows)
        && error.kind() == io::ErrorKind::NotFound
        && matches!(program, "npm" | "pnpm" | "yarn" | "bun" | "npx")
}

#[cfg(test)]
mod tests {
    use super::should_try_windows_cmd_shim;
    use crate::model::{CommandSpec, PackageManager};
    use std::io;

    #[test]
    fn builds_package_manager_commands_without_shell() {
        assert_eq!(
            PackageManager::Pnpm.script_command("dev").display(),
            "pnpm run dev"
        );
        assert_eq!(
            PackageManager::Npm.script_command("dev").display(),
            "npm run dev"
        );
        assert_eq!(
            PackageManager::Bun.script_command("dev").display(),
            "bun run dev"
        );
        assert_eq!(
            PackageManager::Yarn.script_command("dev").display(),
            "yarn dev"
        );
    }

    #[test]
    fn displays_program_without_args() {
        assert_eq!(
            CommandSpec::new("srun", std::iter::empty::<&str>()).display(),
            "srun"
        );
    }

    #[test]
    fn windows_shim_fallback_only_applies_to_package_managers() {
        let error = io::Error::new(io::ErrorKind::NotFound, "missing");

        assert_eq!(should_try_windows_cmd_shim("npm", &error), cfg!(windows));
        assert!(!should_try_windows_cmd_shim("cargo", &error));
    }
}
