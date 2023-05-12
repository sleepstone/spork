mod error;
mod project;

use std::env::current_dir;

use clap::{Parser, Subcommand};
use error::{FatalError, FatalResult};
use project::{new_project, ProjectType};

#[macro_export]
macro_rules! success {
    ($($arg:tt)+) => {{
        println!("{} {}", yansi::Paint::green("[*]").bold(), format!($($arg)+))
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
    },

    /// Create a new spork project in the current directory
    Init {
        /// Create a library project instead of an executable
        #[arg(short, long)]
        lib: bool,
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
        Commands::New { name, lib } => {
            let project_type = if lib {
                ProjectType::Library
            } else {
                ProjectType::Executable
            };

            new_project(&name, &name, project_type)?
        }

        Commands::Init { lib } => {
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

            new_project(project_name, project_path, project_type)?;
        }
    };

    Ok(())
}
