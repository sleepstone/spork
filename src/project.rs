use std::fs;

use serde::{Deserialize, Serialize};

use crate::error::{FatalError, FatalResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct BuildFile {
    pub project: ProjectInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectInfo {
    pub name: String,
}

pub fn parse_spork_file(path: &str) -> FatalResult<BuildFile> {
    let toml_src = match fs::read_to_string(path) {
        Ok(res) => res,
        Err(_) => return Err(FatalError::NoSporkProject),
    };

    match toml::from_str(&toml_src) {
        Ok(res) => Ok(res),
        Err(err) => Err(FatalError::BuildFileParseError { err }),
    }
}
