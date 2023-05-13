use git2::Repository;
use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    project::{BuildFile, ProjectInfo, ProjectType},
    success,
    util::{mkdir, mkfile},
    SPORK_FILE_NAME,
};

pub fn new_project(name: &str, path: &str, project_type: ProjectType) -> FatalResult<()> {
    check_project_name(name)?;

    mkdir(path)?;
    mkdir(&format!("{path}/src"))?;

    if project_type == ProjectType::executable {
        let template_src = include_str!("../template/main.c");
        mkfile(&format!("{path}/src/main.c"), template_src)?;
    }

    let clang_format_src = include_str!("../template/.clang-format");
    mkfile(&format!("{path}/.clang-format"), clang_format_src)?;

    mkfile(&format!("{path}/.gitignore"), ".vscode\nbin\n")?;

    if project_type == ProjectType::library {
        mkdir(&format!("{path}/include"))?;
        mkdir(&format!("{path}/include/{name}"))?;

        let template_header = "#pragma once\n";
        mkfile(&format!("{path}/include/{name}/entry.h"), template_header)?;
    }

    create_spork_file(name, path, project_type)?;

    if let Err(err) = Repository::init(path) {
        return Err(FatalError::FailedRunGitInit { err });
    }

    success!("created {project_type} project '{name}'");

    Ok(())
}

fn create_spork_file(name: &str, path: &str, project_type: ProjectType) -> FatalResult<()> {
    let info_template = BuildFile {
        project: ProjectInfo {
            name: name.to_string(),
            kind: project_type,
        },
    };

    mkfile(
        &format!("{path}/{SPORK_FILE_NAME}"),
        &toml::to_string_pretty(&info_template).unwrap(),
    )?;

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
