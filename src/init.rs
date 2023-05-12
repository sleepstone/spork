use std::{fmt::Display, fs, path::Path};

use git2::Repository;
use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    success,
};

#[derive(PartialEq)]
pub enum ProjectType {
    Executable, // .exe
    Library,    // .dll, .so
}

impl Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Executable => write!(f, "executable"),
            Self::Library => write!(f, "library"),
        }
    }
}

pub fn new_project(name: &str, path: &str, project_type: ProjectType) -> FatalResult<()> {
    check_project_name(name)?;

    mkdir(path)?;
    mkdir(&format!("{path}/src"))?;

    if project_type == ProjectType::Executable {
        let template_src = include_str!("../template/main.c");
        mkfile(&format!("{path}/src/main.c"), template_src)?;
    }

    let clang_format_src = include_str!("../template/.clang-format");
    mkfile(&format!("{path}/.clang-format"), clang_format_src)?;

    if project_type == ProjectType::Library {
        mkdir(&format!("{path}/include"))?;
        mkdir(&format!("{path}/include/{name}"))?;

        let template_header = "#pragma once\n";
        mkfile(&format!("{path}/include/{name}/entry.h"), template_header)?;
    }

    if let Err(err) = Repository::init(path) {
        return Err(FatalError::FailedRunGitInit { err });
    }

    success!("created {project_type} project '{name}'");

    Ok(())
}

fn check_project_name(name: &str) -> FatalResult<()> {
    let verifier = Regex::new(r"[^a-z_]").unwrap();
    if verifier.is_match(name) {
        Err(FatalError::InvalidProjectName)
    } else {
        Ok(())
    }
}

fn mkfile(path: &str, contents: &str) -> FatalResult<()> {
    if let Err(err) = fs::write(path, contents) {
        Err(FatalError::CannotCreateFile {
            path: path.to_string(),
            err,
        })
    } else {
        Ok(())
    }
}

fn mkdir(path: &str) -> FatalResult<()> {
    match Path::new(path).try_exists() {
        Ok(exists) => {
            if exists {
                return Ok(());
            }
        }
        Err(err) => {
            return Err(FatalError::CannotCreateDir {
                path: path.to_string(),
                err,
            });
        }
    }

    if let Err(err) = fs::create_dir(path) {
        Err(FatalError::CannotCreateDir {
            path: path.to_string(),
            err,
        })
    } else {
        Ok(())
    }
}
