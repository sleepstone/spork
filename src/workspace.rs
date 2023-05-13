use std::{collections::HashMap, fs};

use serde::{Deserialize, Serialize};

use crate::error::{FatalError, FatalResult};

#[derive(Serialize, Deserialize, Debug)]
pub struct WorkspaceFile {
    pub workspace: HashMap<String, String>,
}

pub fn parse_spork_workspace(path: &str) -> FatalResult<WorkspaceFile> {
    let toml_src = match fs::read_to_string(path) {
        Ok(res) => res,
        Err(_) => panic!(),
    };

    match toml::from_str(&toml_src) {
        Ok(res) => Ok(res),
        Err(err) => Err(FatalError::BuildFileParseError { err }),
    }
}
