use std::{fmt::Display, io, path::Path};

use crate::{targets::Target, SPORK_FILE_NAME};

pub type FatalResult<T> = Result<T, FatalError>;

#[derive(Debug)]
pub enum FatalError {
    InvalidProjectName,
    CannotCreateFile { path: String, err: io::Error },
    CannotCreateDir { path: String, err: io::Error },
    CannotGetCurrentDir { err: io::Error },
    CannotReadFileInDir { path: String, err: io::Error },
    CannotReadDir { path: String, err: io::Error },
    CannotRemoveDir { path: String, err: io::Error },
    CurrentDirInvalid,
    CurrentDirInvalidUTF8,
    FileInvalidUTF8 { path: Box<Path> },
    FailedRunGitInit { err: git2::Error },
    FailedRunZigcc { err: io::Error },
    FailedRunOutput { path: String, err: io::Error },
    BuildFileParseError { err: toml::de::Error },
    CompilationFailed,
    LinkFailed,
    CannotRunLib,
    NoSporkProject,
    NoSourceFiles,
    NoSupportedTargets,
    BadTarget { target: String },
    InvalidTargetArch { arch: String },
    InvalidTargetOS { os: String },
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
            Self::CannotReadFileInDir { path, err } => {
                write!(f, "couldn't read file in directory '{path}': {err}")
            }
            Self::CannotReadDir { path, err } => {
                write!(f, "couldn't read directory '{path}': {err}")
            }
            Self::CannotRemoveDir { path, err } => {
                write!(f, "couldn't remove directory '{path}': {err}")
            }
            Self::CurrentDirInvalid => write!(f, "current directory is invalid"),
            Self::CurrentDirInvalidUTF8 => {
                write!(f, "current directory contains invalid UTF-8 encoded text")
            }
            Self::FileInvalidUTF8 { path } => {
                write!(
                    f,
                    "file '{}' contains invalid UTF-8 encoded text",
                    path.display()
                )
            }
            Self::FailedRunGitInit { err } => {
                write!(f, "failed to initialize a git repository: {err}")
            }
            Self::FailedRunZigcc { err } => {
                write!(f, "failed to run 'zig cc': {err}")
            }
            Self::FailedRunOutput { path, err } => write!(f, "failed to run '{path}': {err}"),
            Self::BuildFileParseError { err } => {
                writeln!(f, "failed to parse '{SPORK_FILE_NAME}':")?;
                write!(f, "{err}")
            }
            Self::CompilationFailed => write!(f, "compilation failed"),
            Self::LinkFailed => write!(f, "linking failed"),
            Self::CannotRunLib => write!(
                f,
                "only executable projects can be run (use 'spork build' instead)"
            ),
            Self::NoSporkProject => write!(
                f,
                "couldn't find a spork project here - use 'spork new' or 'spork init' to create one"
            ),
            Self::NoSourceFiles => write!(f, "project has no source files"),
            Self::NoSupportedTargets => write!(
                f,
                "unable to run - built target does not match host target of '{}'",
                Target::host().unwrap()
            ),
            Self::BadTarget { target } => {
                write!(f, "target '{target}' must follow format 'arch-os'")
            }
            Self::InvalidTargetArch { arch } => {
                write!(f, "target architecture '{arch}' is invalid")
            }
            Self::InvalidTargetOS { os } => write!(f, "target os '{os}' is invalid"),
        }
    }
}
