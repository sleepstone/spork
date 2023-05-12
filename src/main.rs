mod error;
mod project;

use clap::{Parser, Subcommand};
use error::FatalResult;
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

        /// Whether to create an executable project or a library project
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
        Commands::New { name, lib: library } => {
            let project_type = if library {
                ProjectType::Library
            } else {
                ProjectType::Executable
            };

            new_project(&name, project_type)?
        }
    };

    Ok(())
}
