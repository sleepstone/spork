use std::{fmt::Display, process::Command};

use crate::error::{FatalError, FatalResult};

#[derive(Debug, PartialEq, Clone)]
pub struct Target {
    pub arch: Architecture,
    pub os: OperatingSystem,
}

impl Target {
    pub fn new(target: &str, lenient: bool) -> FatalResult<Self> {
        let mut triple_comps: Vec<_> = target.split("-").collect();

        if !lenient {
            if triple_comps.len() != 2 {
                return Err(FatalError::BadTarget {
                    target: target.to_string(),
                });
            }
        } else {
            triple_comps.remove(1);
        }

        Ok(Target {
            arch: Architecture::new(triple_comps[0])?,
            os: OperatingSystem::new(triple_comps[1])?,
        })
    }

    pub fn host() -> FatalResult<Self> {
        match Command::new("zig").args(["cc", "-dumpmachine"]).output() {
            Ok(output) => {
                let output = String::from_utf8_lossy(&output.stdout);
                Ok(Target::new(&output, true)?)
            }
            Err(err) => Err(FatalError::FailedRunZigcc { err }),
        }
    }

    pub fn ziggified(&self) -> String {
        format!("{}-{}", self.arch, self.os.ziggified())
    }
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.arch, self.os)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Architecture {
    X86,
    X86_64,
}

impl Architecture {
    pub fn new(arch: &str) -> FatalResult<Self> {
        Ok(match arch {
            "x86" => Self::X86,
            "x86_64" => Self::X86_64,
            _ => {
                return Err(FatalError::InvalidTargetArch {
                    arch: arch.to_string(),
                })
            }
        })
    }
}

impl Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::X86 => "x86",
                Self::X86_64 => "x86_64",
            }
        )
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OperatingSystem {
    Freestanding,
    Windows,
    Linux,
}

impl OperatingSystem {
    pub fn new(os: &str) -> FatalResult<Self> {
        Ok(match os {
            "freestanding" => Self::Freestanding,
            "windows" => Self::Windows,
            "linux" => Self::Linux,
            _ => return Err(FatalError::InvalidTargetOS { os: os.to_string() }),
        })
    }

    fn ziggified(&self) -> &'static str {
        match self {
            Self::Freestanding => "freestanding-none",
            Self::Linux => "linux-gnu",
            Self::Windows => "windows-gnu",
        }
    }
}

impl Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Freestanding => "freestanding",
                Self::Windows => "windows",
                Self::Linux => "linux",
            }
        )
    }
}
