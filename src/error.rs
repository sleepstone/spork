use std::{fmt::Display, io};

use yansi::Paint;

pub type FatalResult<T> = Result<T, FatalError>;

pub enum FatalError {
    InvalidProjectName,
    CannotCreateFile { path: String, err: io::Error },
    CannotCreateDir { path: String, err: io::Error },
    FailedRunGitInit { err: git2::Error },
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
            Self::FailedRunGitInit { err } => {
                write!(f, "failed to initialize a git repository: {err}")
            }
        }
    }
}
