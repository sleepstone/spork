use std::{
    env,
    fs::{self, DirEntry},
    path::Path,
};

use regex::Regex;

use crate::error::{FatalError, FatalResult};

pub fn mkfile(path: &str, contents: &str) -> FatalResult<()> {
    if let Err(err) = fs::write(path, contents) {
        Err(FatalError::CannotCreateFile {
            path: path.to_string(),
            err,
        })
    } else {
        Ok(())
    }
}

pub fn mkdir(path: &str) -> FatalResult<()> {
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

pub fn mkdir_all(path: &str) -> FatalResult<()> {
    if let Err(err) = fs::create_dir_all(path) {
        Err(FatalError::CannotCreateDir {
            path: path.to_string(),
            err,
        })
    } else {
        Ok(())
    }
}

pub fn walkdir(path: &str) -> FatalResult<Vec<String>> {
    let mut output = Vec::new();

    let path_dir = match fs::read_dir(path) {
        Ok(res) => res,
        Err(err) => {
            return Err(FatalError::CannotReadDir {
                path: path.to_string(),
                err,
            })
        }
    };

    for entry in path_dir {
        let entry = match entry {
            Ok(res) => res,
            Err(err) => {
                return Err(FatalError::CannotReadFileInDir {
                    path: path.to_string(),
                    err,
                })
            }
        };

        let entry_metadata = match entry.metadata() {
            Ok(res) => res,
            Err(err) => {
                return Err(FatalError::CannotReadFileInDir {
                    path: path.to_string(),
                    err,
                })
            }
        };

        if entry_metadata.is_dir() {
            output.append(&mut walkdir(&entry_to_string(&entry)?)?);
        } else {
            output.push(entry_to_string(&entry)?);
        }
    }

    Ok(output)
}

fn entry_to_string(entry: &DirEntry) -> FatalResult<String> {
    match entry.path().to_str() {
        Some(res) => Ok(res.to_string()),
        None => Err(FatalError::FileInvalidUTF8 {
            path: entry.path().into(),
        }),
    }
}

pub fn check_project_name(name: &str) -> FatalResult<()> {
    let verifier = Regex::new(r"[^a-z_]").unwrap();
    if verifier.is_match(name) {
        Err(FatalError::InvalidProjectName)
    } else {
        Ok(())
    }
}

pub static mut LAUNCH_DIR: String = String::new();

pub fn update_launch_dir() {
    unsafe { LAUNCH_DIR = env::current_dir().unwrap().to_string_lossy().to_string() }
}
