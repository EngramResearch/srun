use crate::detect::detect_project;
use crate::exec::run_command;
use crate::model::{Intent, ProjectInfo, ResolvedCommand, SrunError};
use crate::resolve::resolve_intent;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "srun", about = "Universal Smart Project Runner")]
pub struct Cli {
    #[arg(long, global = true, default_value = ".")]
    pub root: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Dev(RunArgs),
    Build(RunArgs),
    Installer(RunArgs),
    Test(RunArgs),
    #[command(alias = "typecheck", alias = "type-check")]
    Check(RunArgs),
    Preview(RunArgs),
    Clean(RunArgs),
    #[command(alias = "install", alias = "bootstrap")]
    Setup(RunArgs),
    Lint(RunArgs),
    Format(RunArgs),
    Info,
    List,
}

#[derive(Debug, Args, Clone, Copy)]
pub struct RunArgs {
    #[arg(long)]
    pub dry_run: bool,

    #[arg(long)]
    pub verbose: bool,
}

pub fn run(cli: Cli) -> Result<i32, SrunError> {
    let project = detect_project(&cli.root)?;

    match cli.command {
        Command::Info => {
            print_info(&project);
            Ok(0)
        }
        Command::List => {
            print_list(&project);
            Ok(0)
        }
        Command::Dev(args) => run_intent(&project, Intent::Dev, args),
        Command::Build(args) => run_intent(&project, Intent::Build, args),
        Command::Installer(args) => run_intent(&project, Intent::Installer, args),
        Command::Test(args) => run_intent(&project, Intent::Test, args),
        Command::Check(args) => run_intent(&project, Intent::Check, args),
        Command::Preview(args) => run_intent(&project, Intent::Preview, args),
        Command::Clean(args) => run_intent(&project, Intent::Clean, args),
        Command::Setup(args) => run_intent(&project, Intent::Setup, args),
        Command::Lint(args) => run_intent(&project, Intent::Lint, args),
        Command::Format(args) => run_intent(&project, Intent::Format, args),
    }
}

fn run_intent(project: &ProjectInfo, intent: Intent, args: RunArgs) -> Result<i32, SrunError> {
    if args.verbose {
        print_detect(project);
    }

    let resolved = resolve_intent(project, intent)?;

    if args.verbose {
        println!("[resolve]");
        println!("selected: {}", resolved.command.display());
        println!("reason: {}", resolved.reason);
        println!();
    }

    if args.dry_run {
        println!("{}", resolved.command.display());
        return Ok(0);
    }

    if args.verbose {
        println!("[exec]");
        println!("{}", resolved.command.display());
        println!();
    }

    let status = run_command(&resolved.command, &project.root)?;
    Ok(status.code().unwrap_or(1))
}

pub fn print_info(project: &ProjectInfo) {
    println!("Project Type: {}", project.project_type());
    println!(
        "Package Manager: {}",
        project
            .package_manager
            .map(|package_manager| package_manager.label())
            .unwrap_or("none")
    );

    if !project.warnings.is_empty() {
        println!();
        println!("Warnings:");
        for warning in &project.warnings {
            println!("  - {}", warning);
        }
    }

    println!();
    println!("Resolved Commands:");
    for intent in Intent::EXECUTABLE {
        print_resolved_line(project, intent);
    }
}

pub fn print_list(project: &ProjectInfo) {
    println!("Available intents:");
    for intent in Intent::EXECUTABLE {
        match resolve_intent(project, intent) {
            Ok(resolved) => println!("  {:<10} {}", intent.label(), resolved.command.display()),
            Err(_) => println!("  {:<10} unavailable", intent.label()),
        }
    }

    let scripts = project.scripts();
    if !scripts.is_empty() {
        println!();
        println!("Package scripts:");
        for (name, script) in scripts {
            println!("  {:<20} {}", name, script);
        }
    }
}

fn print_resolved_line(project: &ProjectInfo, intent: Intent) {
    match resolve_intent(project, intent) {
        Ok(resolved) => println!("{}:\n  {}", intent.label(), resolved.command.display()),
        Err(error) => println!("{}:\n  unavailable ({})", intent.label(), error),
    }
}

fn print_detect(project: &ProjectInfo) {
    println!("[detect]");
    for trace in &project.traces {
        println!("{}", trace);
    }
    for warning in &project.warnings {
        println!("warning: {}", warning);
    }
    println!("project type: {}", project.project_type());
    println!();
}

#[allow(dead_code)]
fn _format_resolved(resolved: &ResolvedCommand) -> String {
    format!(
        "{} -> {}",
        resolved.intent.label(),
        resolved.command.display()
    )
}
