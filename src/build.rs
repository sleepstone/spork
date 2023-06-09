use std::{
    collections::{hash_map::IntoIter, HashMap},
    env,
    fmt::Display,
    process::Command,
    time::Instant,
};

use regex::Regex;

use crate::{
    error::{FatalError, FatalResult},
    progress,
    project::{parse_spork_file, ProjectFile, ProjectType},
    success,
    targets::{OperatingSystem, Target},
    util::{mkdir_all, walkdir, LAUNCH_DIR},
    warning, SPORK_FILE_NAME,
};

pub struct BuildInfo {
    pub name: String,
    pub release: bool,
    pub kind: ProjectType,
    pub target: Target,
    pub output_path: Option<String>,
    pub dependencies: Option<Dependencies>,
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
    build_project(spork_file, release, all)
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
                let target = Target::new(&target, false)?;
                let dependencies = match &spork_file.project.dependencies {
                    Some(deps) => Some(Dependencies::new(deps.clone(), &target)?),
                    None => None,
                };

                let mut info = BuildInfo {
                    name: spork_file.project.name.clone(),
                    release,
                    kind: spork_file.project.kind,
                    target,
                    output_path: None,
                    dependencies,
                };

                build_target(&mut info)?;
                build_infos.push(info);
            }
        } else {
            let target = Target::new(&targets[0], false)?;
            let dependencies = match spork_file.project.dependencies {
                Some(deps) => Some(Dependencies::new(deps, &target)?),
                None => None,
            };

            let mut info = BuildInfo {
                name: spork_file.project.name.clone(),
                release,
                kind: spork_file.project.kind,
                target,
                output_path: None,
                dependencies,
            };

            build_target(&mut info)?;
            build_infos.push(info);
        }
    } else {
        let target = Target::host()?;
        let dependencies = match spork_file.project.dependencies {
            Some(deps) => Some(Dependencies::new(deps, &target)?),
            None => None,
        };

        let mut info = BuildInfo {
            name: spork_file.project.name,
            release,
            kind: spork_file.project.kind,
            target,
            output_path: None,
            dependencies,
        };

        build_target(&mut info)?;
        build_infos.push(info);
    }

    Ok(build_infos)
}

fn build_target(info: &mut BuildInfo) -> FatalResult<()> {
    let current_dir = match env::current_dir() {
        Ok(res) => res.to_string_lossy().to_string(),
        Err(err) => return Err(FatalError::CouldntGetWorkDir { err }),
    };

    if let Some(deps) = info.dependencies.clone() {
        for (dep_path, dep) in deps {
            if let Err(err) = env::set_current_dir(&dep_path) {
                return Err(FatalError::CouldntChangeWorkDir { dir: dep_path, err });
            }

            build_target(&mut BuildInfo {
                name: dep.name,
                release: info.release,
                kind: ProjectType::library,
                target: info.target.clone(),
                output_path: info.output_path.clone(),
                dependencies: dep.deps,
            })?;

            if let Err(err) = env::set_current_dir(&current_dir) {
                return Err(FatalError::CouldntChangeWorkDir {
                    dir: current_dir,
                    err,
                });
            }
        }
    }

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
    cmd.args(info.target.cc_args());

    if info.kind == ProjectType::library {
        cmd.args(["-Iinclude", "-DSPORK_EXPORT"]);
    } else if let Some(deps) = info.dependencies.clone() {
        for (_, dep) in deps {
            cmd.arg(format!("-I../{}/include", dep.name));
        }
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
    cmd.args([&format!("-L{}", out_dir(info))]);
    cmd.args(objects);

    if info.kind == ProjectType::library {
        cmd.args(["-shared"]);

        if info.target.os == OperatingSystem::Windows {
            let import_lib_path = output_path.replace(".dll", ".lib");
            cmd.arg(&format!("-Wl,--out-implib,{}", import_lib_path));
            cmd.args(["-o", output_path]);
        } else {
            cmd.args([
                "-o",
                &output_path.replace(&info.name, &format!("lib{}", info.name)),
            ]);
        }
    } else {
        if let Some(deps) = info.dependencies.clone() {
            if info.target.os != OperatingSystem::Windows {
                cmd.arg(format!("-Wl,-rpath,{}", "."));
            }

            for (_, dep) in deps {
                cmd.arg(format!("-l{}", dep.name));
            }
        }

        cmd.args(["-o", output_path]);
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
    let prefix = unsafe { format!("{LAUNCH_DIR}/bin/{}", info.target) };

    if info.release {
        format!("{prefix}/release")
    } else {
        format!("{prefix}/debug")
    }
}

#[derive(Clone)]
pub struct Dependency {
    name: String,
    deps: Option<Dependencies>,
}

#[derive(Clone)]
pub struct Dependencies {
    path_to_deps: HashMap<String, Dependency>,
}

impl Dependencies {
    pub fn new(paths: Vec<String>, target: &Target) -> FatalResult<Self> {
        let mut path_to_deps = HashMap::new();

        for path in paths {
            let spork_file = parse_spork_file(&format!("{path}/{SPORK_FILE_NAME}"))?;

            if spork_file.project.kind != ProjectType::library {
                return Err(FatalError::NoExecutableDependencies {
                    name: spork_file.project.name,
                });
            }

            if let Some(targets) = spork_file.project.targets {
                if !targets.contains(&target.to_string()) {
                    return Err(FatalError::NoTargetSupportDependency {
                        dep: spork_file.project.name,
                        target: target.clone(),
                    });
                }
            }

            path_to_deps.insert(
                path,
                Dependency {
                    name: spork_file.project.name,
                    deps: match spork_file.project.dependencies {
                        Some(deps) => Some(Dependencies::new(deps, target)?),
                        None => None,
                    },
                },
            );
        }

        Ok(Self { path_to_deps })
    }
}

impl IntoIterator for Dependencies {
    type Item = (String, Dependency);

    type IntoIter = IntoIter<String, Dependency>;

    fn into_iter(self) -> Self::IntoIter {
        self.path_to_deps.into_iter()
    }
}
