use std::{fmt::Display, io};

use yansi::Paint;

pub type FatalResult<T> = Result<T, FatalError>;

pub enum FatalError {
    InvalidProjectName,
    CannotCreateFile { path: String, err: io::Error },
    CannotCreateDir { path: String, err: io::Error },
    CannotGetCurrentDir { err: io::Error },
    CannotReadDir { path: String, err: io::Error },
    CurrentDirInvalid,
    CurrentDirInvalidUTF8,
    FailedRunGitInit { err: git2::Error },
    BuildFileParseError { err: toml::de::Error },
}

impl FatalError {
    pub fn print(&self) {
        println!("{} {}", Paint::red("[!]").bold(), self);
    }
}

impl Display for FatalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProjectName => write!(
                f,
                "project names can only contain lowercase ASCII letters and underscores"
            ),
            Self::CannotCreateFile { path, err } => {
                write!(f, "cannot create file '{path}': {err}")
            }
            Self::CannotCreateDir { path, err } => {
                write!(f, "cannot create directory at '{path}': {err}")
            }
            Self::CannotGetCurrentDir { err } => write!(f, "couldn't get current directory: {err}"),
            Self::CannotReadDir { path, err } => {
                write!(f, "couldn't read directory '{path}': {err}")
            }
            Self::CurrentDirInvalid => write!(f, "current directory is invalid"),
            Self::CurrentDirInvalidUTF8 => {
                write!(f, "current directory contains invalid UTF-8 encoded text")
            }
            Self::FailedRunGitInit { err } => {
                write!(f, "failed to initialize a git repository: {err}")
            }
            Self::BuildFileParseError { err } => {
                writeln!(f, "failed to parse Spork.toml file:")?;
                write!(f, "{err}")
            }
        }
    }
}
