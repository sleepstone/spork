use std::{fmt::Display, process::Command, time::Instant};

use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    progress,
    project::{parse_spork_file, ProjectType},
    success,
    util::{mkdir_all, walkdir},
    warning, SPORK_FILE_NAME,
};

pub struct BuildInfo {
    pub release: bool,
    pub kind: ProjectType,
}

impl Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.release {
            write!(f, "{}, release", self.kind)?;
        } else {
            write!(f, "{}, debug", self.kind)?;
        }

        Ok(())
    }
}

pub fn build(release: bool) -> FatalResult<(String, BuildInfo)> {
    let obj_name_regex = Regex::new(r"[/\\]").unwrap();

    let start_time = Instant::now();
    let spork_file = parse_spork_file(SPORK_FILE_NAME)?;

    let info = BuildInfo {
        release,
        kind: spork_file.project.kind,
    };

    let out_dir = out_dir(&info);
    progress!("building '{}'...", spork_file.project.name);

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
        let build_obj_result = build_obj(&file, &obj_path, &info)?;
        if !build_obj_result {
            had_error = true;
        }

        objects.push(obj_path);
    }

    if objects.is_empty() {
        return Err(FatalError::NoSourceFiles);
    }

    let output_path = match info.kind {
        ProjectType::executable => format!("{out_dir}/{}.exe", spork_file.project.name),
        ProjectType::library => format!("{out_dir}/{}.dll", spork_file.project.name),
    };

    if !had_error {
        let build_output_res = build_output(objects, &output_path, &info)?;

        if build_output_res {
            let end_time = Instant::now();
            success!("finished in {:.2?} ({info})", end_time - start_time);
        } else {
            return Err(FatalError::LinkFailed);
        }
    } else {
        return Err(FatalError::CompilationFailed);
    }

    Ok((output_path, info))
}

pub fn build_and_run(release: bool) -> FatalResult<()> {
    let (output, info) = build(release)?;

    if info.kind == ProjectType::library {
        return Err(FatalError::CannotRunLib);
    }

    if let Err(err) = Command::new(&output).status() {
        Err(FatalError::FailedRunOutput { path: output, err })
    } else {
        Ok(())
    }
}

fn build_obj(src_path: &str, obj_path: &str, info: &BuildInfo) -> FatalResult<bool> {
    let mut cmd = common_build_cmd();
    cmd.args(["-c", src_path, "-o", obj_path, "-Isrc"]);

    if info.kind == ProjectType::library {
        cmd.args(["-Iinclude"]);
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
    let mut cmd = common_build_cmd();
    cmd.args(["-o", output_path]);
    cmd.args(objects);

    if info.kind == ProjectType::library {
        let import_lib_path = output_path.replace(".dll", ".lib");

        cmd.args([
            "-shared",
            "-DSPORK_EXPORT",
            &format!("-Wl,--out-implib,{}", import_lib_path),
        ]);
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

fn common_build_cmd() -> Command {
    let mut cmd = Command::new("zig");
    cmd.args(["cc", "-std=c17", "-Wall", "-Wextra", "-Wpedantic"]);

    cmd
}

fn out_dir(info: &BuildInfo) -> String {
    if info.release {
        String::from("bin/release")
    } else {
        String::from("bin/debug")
    }
}
