mod build;
mod error;
mod init;
mod project;
mod targets;
mod util;
mod workspace;

use std::{env::current_dir, fs, process::exit};

use clap::{Parser, Subcommand};
use error::{FatalError, FatalResult};
use project::{parse_spork_file, ProjectType};

const SPORK_FILE_NAME: &str = "Spork.toml";

#[macro_export]
macro_rules! success {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::green("[*]").bold(), format!($($arg)+))
    }};
}

#[macro_export]
macro_rules! progress {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::blue("[+]").bold(), format!($($arg)+))
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::yellow("[?]").bold(), format!($($arg)+))
    }};
}

#[macro_export]
macro_rules! fatal_error {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::red("[!]").bold(), format!($($arg)+))
    }};
}

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new spork project
    New {
        /// Name of the project
        name: String,

        /// Create a library project instead of an executable
        #[arg(short, long)]
        lib: bool,

        /// Create project even if directory already contains files
        #[arg(short, long)]
        force: bool,
    },

    /// Create a new spork project in the current directory
    Init {
        /// Create a library project instead of an executable
        #[arg(short, long)]
        lib: bool,

        /// Create project even if directory already contains files
        #[arg(short, long)]
        force: bool,
    },

    /// Build the current project
    Build {
        /// Build in release mode instead of debug
        #[arg(short, long)]
        release: bool,

        /// Build for all targets
        #[arg(short, long)]
        all: bool,
    },

    /// Build and run the current project
    Run {
        /// Build in release mode instead of debug
        #[arg(short, long)]
        release: bool,

        /// Build for all targets
        #[arg(short, long)]
        all: bool,
    },

    /// Removes the 'bin' directory
    Clean,
}

fn main() {
    if let Err(err) = init() {
        fatal_error!("{err}");
        exit(1);
    }
}

fn init() -> FatalResult<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::New { name, lib, force } => new_project(&name, lib, force),
        Commands::Init { lib, force } => init_project(lib, force),

        Commands::Build { release, all } => build_project(release, all),
        Commands::Run { release, all } => run_project(release, all),
        Commands::Clean => clean_project(),
    }
}

fn new_project(name: &str, lib: bool, force: bool) -> FatalResult<()> {
    let project_type = if lib {
        ProjectType::library
    } else {
        ProjectType::executable
    };

    if let Ok(dir) = fs::read_dir(name) {
        if dir.count() != 0 && !force {
            warning!("directory is not empty: use --force to override");
            return Ok(());
        }
    }

    init::new_project(name, name, project_type)?;

    Ok(())
}

fn init_project(lib: bool, force: bool) -> FatalResult<()> {
    let project_type = if lib {
        ProjectType::library
    } else {
        ProjectType::executable
    };

    let current_dir = match current_dir() {
        Ok(res) => res,
        Err(err) => return Err(FatalError::CannotGetCurrentDir { err }),
    };

    let project_name = match current_dir.iter().last() {
        Some(res) => match res.to_str() {
            Some(res) => res,
            None => return Err(FatalError::CurrentDirInvalidUTF8),
        },
        None => return Err(FatalError::CurrentDirInvalid),
    };

    let project_path = match current_dir.to_str() {
        Some(res) => res,
        None => return Err(FatalError::CurrentDirInvalidUTF8),
    };

    // Check if it is empty
    match fs::read_dir(project_path) {
        Ok(res) => {
            if res.count() != 0 && !force {
                warning!("directory is not empty: use --force to override");
                return Ok(());
            }
        }
        Err(err) => {
            return Err(FatalError::CannotReadDir {
                path: project_path.to_string(),
                err,
            })
        }
    }

    init::new_project(project_name, project_path, project_type)?;

    Ok(())
}

fn build_project(release: bool, all: bool) -> FatalResult<()> {
    build::build(release, all)?;
    Ok(())
}

fn run_project(release: bool, all: bool) -> FatalResult<()> {
    build::build_and_run(release, all)?;
    Ok(())
}

fn clean_project() -> FatalResult<()> {
    parse_spork_file(SPORK_FILE_NAME)?;
    if let Err(err) = fs::remove_dir_all("bin") {
        return Err(FatalError::CannotRemoveDir {
            path: String::from("bin"),
            err,
        });
    }
    success!("cleaned");

    Ok(())
}
