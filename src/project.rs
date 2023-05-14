use std::{fmt::Display, fs};

use serde::{Deserialize, Serialize};

use crate::error::{FatalError, FatalResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectFile {
    pub project: ProjectInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub kind: ProjectType,
    pub targets: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

pub fn parse_spork_file(path: &str) -> FatalResult<ProjectFile> {
    let toml_src = match fs::read_to_string(path) {
        Ok(res) => res,
        Err(_) => {
            return Err(FatalError::NoSporkToml {
                path: path.to_string(),
            })
        }
    };

    let project_file: ProjectFile = match toml::from_str(&toml_src) {
        Ok(res) => res,
        Err(err) => {
            return Err(FatalError::BuildFileParseError { err });
        }
    };

    Ok(project_file)
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub enum ProjectType {
    executable, // .exe
    library,    // .dll, .so
}

impl Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::executable => write!(f, "executable"),
            Self::library => write!(f, "library"),
        }
    }
}
