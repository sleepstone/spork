use std::{process::Command, time::Instant};

use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    progress,
    project::parse_spork_file,
    success,
    util::{mkdir, walkdir},
    warning, SPORK_FILE_NAME,
};

pub fn build(path: &str) -> FatalResult<String> {
    let obj_name_regex = Regex::new(r"[/\\]").unwrap();

    let start_time = Instant::now();
    let spork_file = parse_spork_file(SPORK_FILE_NAME)?;
    progress!("building '{}'...", spork_file.project.name);

    mkdir(path)?;
    mkdir(&format!("{path}/obj"))?;

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

        let obj_path = format!("bin/obj/{}.o", obj_name);
        let build_obj_result = build_obj(&file, &obj_path)?;
        if !build_obj_result {
            had_error = true;
        }

        objects.push(obj_path);
    }

    let output_path = format!("bin/{}.exe", spork_file.project.name);

    if !had_error {
        let build_exe_result = build_exe(objects, &output_path)?;

        if build_exe_result {
            let end_time = Instant::now();
            success!("finished in {:?}", end_time - start_time);
        } else {
            return Err(FatalError::LinkFailed);
        }
    } else {
        return Err(FatalError::CompilationFailed);
    }

    Ok(output_path)
}

pub fn build_and_run(path: &str) -> FatalResult<()> {
    let output = build(path)?;

    if let Err(err) = Command::new(&output).status() {
        Err(FatalError::FailedRunOutput { path: output, err })
    } else {
        Ok(())
    }
}

fn build_obj(src_path: &str, obj_path: &str) -> FatalResult<bool> {
    let mut cmd = Command::new("zig");
    cmd.args([
        "cc",
        "-std=c17",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-c",
        src_path,
        "-o",
        obj_path,
    ]);

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

fn build_exe(objects: Vec<String>, output_path: &str) -> FatalResult<bool> {
    let mut cmd = Command::new("zig");
    cmd.args([
        "cc",
        "-std=c17",
        "-Wall",
        "-Wextra",
        "-Wpedantic",
        "-o",
        output_path,
    ]);

    cmd.args(objects);

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
