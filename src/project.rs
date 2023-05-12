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
