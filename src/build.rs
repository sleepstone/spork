use std::{fmt::Display, path::Path, process::Command, time::Instant};

use git2::Repository;
use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    progress,
    project::{parse_spork_file, BuildFile, ProjectFile, ProjectType},
    success,
    targets::{OperatingSystem, Target},
    util::{check_member_name, mkdir, mkdir_all, walkdir},
    warning,
    workspace::WorkspaceFile,
    SPORK_FILE_NAME,
};

pub struct BuildInfo {
    pub name: String,
    pub release: bool,
    pub kind: ProjectType,
    pub target: Target,
    pub output_path: Option<String>,
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let suffix = format!(", {}", self.target);

        if self.release {
            write!(f, "{}, release{}", self.kind, suffix)?;
        } else {
            write!(f, "{}, debug{}", self.kind, suffix)?;
        }

        Ok(())
    }
}

pub fn build(release: bool, all: bool) -> FatalResult<Vec<BuildInfo>> {
    let spork_file = parse_spork_file(SPORK_FILE_NAME)?;

    match spork_file {
        BuildFile::Project(proj) => build_project(proj, release, all),
        BuildFile::Workspace(ws) => build_workspace(ws, release, all),
    }
}

pub fn build_and_run(release: bool, all: bool) -> FatalResult<()> {
    let infos = build(release, all)?;
    let mut has_run = false;

    for info in infos {
        if info.kind == ProjectType::library {
            return Err(FatalError::CannotRunLib);
        }

        let host = Target::host()?;
        if info.target != host {
            continue;
        }

        let output_to_run = info.output_path.unwrap();
        if let Err(err) = Command::new(&output_to_run).status() {
            return Err(FatalError::FailedRunOutput {
                path: output_to_run,
                err,
            });
        }
        has_run = true;
    }

    if has_run {
        Ok(())
    } else {
        Err(FatalError::NoSupportedTargets)
    }
}

fn build_project(spork_file: ProjectFile, release: bool, all: bool) -> FatalResult<Vec<BuildInfo>> {
    let mut build_infos = Vec::new();
    if let Some(targets) = spork_file.project.targets {
        if targets.is_empty() {
            warning!("no targets specified - nothing will be built");
            return Ok(build_infos);
        }

        if all {
            for target in targets {
                let mut info = BuildInfo {
                    name: spork_file.project.name.clone(),
                    release,
                    kind: spork_file.project.kind,
                    target: Target::new(&target, false)?,
                    output_path: None,
                };

                build_target(&mut info)?;
                build_infos.push(info);
            }
        } else {
            let mut info = BuildInfo {
                name: spork_file.project.name.clone(),
                release,
                kind: spork_file.project.kind,
                target: Target::new(&targets[0], false)?,
                output_path: None,
            };

            build_target(&mut info)?;
            build_infos.push(info);
        }
    } else {
        let mut info = BuildInfo {
            name: spork_file.project.name,
            release,
            kind: spork_file.project.kind,
            target: Target::host()?,
            output_path: None,
        };

        build_target(&mut info)?;
        build_infos.push(info);
    }

    Ok(build_infos)
}

fn build_workspace(
    spork_file: WorkspaceFile,
    release: bool,
    all: bool,
) -> FatalResult<Vec<BuildInfo>> {
    let mut build_infos = Vec::new();

    for (member_name, url) in spork_file.workspace {
        check_member_name(&member_name)?;
        let clone_path = Path::new(&member_name);

        if !clone_path.exists() {
            mkdir(&member_name)?;

            if let Err(err) = Repository::clone(&url, clone_path) {
                return Err(FatalError::FailedRunGitClone {
                    path: clone_path.into(),
                    url,
                    err,
                });
            };
        } else {
        }

        let member_spork_file = match parse_spork_file(&format!("{member_name}/{SPORK_FILE_NAME}"))?
        {
            BuildFile::Project(proj) => proj,
            BuildFile::Workspace(_) => return Err(FatalError::NoNestedWorkspaces),
        };

        success!("downloaded '{member_name}'");

        build_infos.append(&mut build_project(member_spork_file, release, all)?);
    }

    Ok(build_infos)
}

fn build_target(info: &mut BuildInfo) -> FatalResult<()> {
    let start_time = Instant::now();

    let obj_name_regex = Regex::new(r"[/\\]").unwrap();

    let out_dir = out_dir(info);
    progress!("building '{}'...", info.name);

    mkdir_all(&format!("{out_dir}/obj"))?;

    let mut objects = Vec::new();
    let mut had_error = false;

    for file in walkdir("src")? {
        if !(file.ends_with(".c") || file.ends_with(".h")) {
            warning!("spork can only compile C files (*.c, *.h) - consider removing file '{file}'");
            continue;
        }

        if file.ends_with(".h") {
            // TODO : proper handling of h files
            continue;
        }

        let obj_name = obj_name_regex.replace_all(&file[4..(file.len() - 2)], "-");

        let obj_path = format!("{out_dir}/obj/{}.o", obj_name);
        let build_obj_result = build_obj(&file, &obj_path, info)?;
        if !build_obj_result {
            had_error = true;
        }

        objects.push(obj_path);
    }

    if objects.is_empty() {
        return Err(FatalError::NoSourceFiles);
    }

    let output_path = match info.kind {
        ProjectType::executable => {
            if info.target.os == OperatingSystem::Windows {
                format!("{out_dir}/{}.exe", info.name)
            } else {
                format!("{out_dir}/{}", info.name)
            }
        }
        ProjectType::library => {
            if info.target.os == OperatingSystem::Windows {
                format!("{out_dir}/{}.dll", info.name)
            } else {
                format!("{out_dir}/{}.so", info.name)
            }
        }
    };

    if !had_error {
        let build_output_res = build_output(objects, &output_path, info)?;

        if build_output_res {
            let end_time = Instant::now();
            success!("finished in {:.2?} ({info})", end_time - start_time);
        } else {
            return Err(FatalError::LinkFailed);
        }
    } else {
        return Err(FatalError::CompilationFailed);
    }

    info.output_path = Some(output_path);

    Ok(())
}

fn build_obj(src_path: &str, obj_path: &str, info: &BuildInfo) -> FatalResult<bool> {
    let mut cmd = common_build_cmd(info);
    cmd.args(["-c", src_path, "-o", obj_path, "-Isrc"]);

    if info.kind == ProjectType::library {
        cmd.args(["-Iinclude", "-DSPORK_EXPORT"]);
    }

    if info.release {
        cmd.args(["-O3", "-s"]);
    } else {
        cmd.args(["-O0", "-g", "-DSPORK_DEBUG"]);
    }

    let cmd_output = match cmd.status() {
        Ok(res) => res,
        Err(err) => return Err(FatalError::FailedRunZigcc { err }),
    };

    if cmd_output.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn build_output(objects: Vec<String>, output_path: &str, info: &BuildInfo) -> FatalResult<bool> {
    let mut cmd = common_build_cmd(info);
    cmd.args(["-o", output_path]);
    cmd.args(objects);

    if info.kind == ProjectType::library {
        cmd.args(["-shared"]);

        if info.target.os == OperatingSystem::Windows {
            let import_lib_path = output_path.replace(".dll", ".lib");
            cmd.arg(&format!("-Wl,--out-implib,{}", import_lib_path));
        }
    }

    if info.release {
        cmd.args(["-O3", "-s"]);
    } else {
        cmd.args(["-O0", "-g"]);
    }

    let cmd_output = match cmd.status() {
        Ok(res) => res,
        Err(err) => return Err(FatalError::FailedRunZigcc { err }),
    };

    if cmd_output.success() {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn common_build_cmd(info: &BuildInfo) -> Command {
    let mut cmd = Command::new("zig");
    cmd.args(["cc", "-std=c17", "-Wall", "-Wextra", "-Wpedantic"]);

    cmd.args(["-target", &info.target.ziggified()]);

    cmd
}

fn out_dir(info: &BuildInfo) -> String {
    let prefix = format!("bin/{}", info.target);

    if info.release {
        format!("{prefix}/release")
    } else {
        format!("{prefix}/debug")
    }
}
