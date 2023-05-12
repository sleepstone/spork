use std::time::Instant;

use crate::{
    error::FatalResult,
    progress,
    project::parse_spork_file,
    success,
    util::{mkdir, walkdir},
    SPORK_FILE_NAME,
};

pub fn build(path: &str) -> FatalResult<()> {
    let start_time = Instant::now();
    let spork_file = parse_spork_file(SPORK_FILE_NAME)?;
    progress!("building '{}'...", spork_file.project.name);
    mkdir(path)?;

    for file in walkdir("src")? {
        println!("{file}");
    }

    let end_time = Instant::now();
    success!("finished in {:?}", end_time - start_time);
    Ok(())
}
