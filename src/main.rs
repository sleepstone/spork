mod error;
mod project;

use std::{env::current_dir, fs};

use clap::{Parser, Subcommand};
use error::{FatalError, FatalResult};
use project::{new_project, ProjectType};

#[macro_export]
macro_rules! success {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::green("[*]").bold(), format!($($arg)+))
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::yellow("[?]").bold(), format!($($arg)+))
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
}

fn main() {
    if let Err(err) = init() {
        err.print();
    }
}

fn init() -> FatalResult<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Commands::New { name, lib, force } => {
            let project_type = if lib {
                ProjectType::Library
            } else {
                ProjectType::Executable
            };

            if let Ok(dir) = fs::read_dir(&name) {
                if dir.count() != 0 && !force {
                    warning!("directory is not empty: use --force to override");
                    return Ok(());
                }
            }

            new_project(&name, &name, project_type)?
        }

        Commands::Init { lib, force } => {
            let project_type = if lib {
                ProjectType::Library
            } else {
                ProjectType::Executable
            };

            let current_dir = match current_dir() {
                Ok(res) => res,
                Err(err) => return Err(FatalError::CannotGetCurrentDir { err }),
            };

            let project_name = match current_dir.into_iter().last() {
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

            new_project(project_name, project_path, project_type)?;
        }
    };

    Ok(())
}
