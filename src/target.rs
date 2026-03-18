use std::str::FromStr;
use std::fmt;
use serde::Deserialize;
use strum_macros::EnumString;

#[derive(Deserialize, Clone, PartialEq, EnumString, Debug)]
pub enum Architecture {
    #[serde(rename = "x86")]
    #[strum(serialize = "x86")]
    X86,
    #[serde(rename = "x86_64")]
    #[strum(serialize = "x86_64")]
    X64,
    #[serde(rename = "arm")]
    #[strum(serialize = "arm")]
    ARM,
    #[serde(rename = "aarch64")]
    #[strum(serialize = "aarch64")]
    ARM64,
    #[serde(rename = "riscv64")]
    #[strum(serialize = "riscv64")]
    RISCV64,
}

//TODO: These may not be the right platforms.. Maybe we'll have to make this more precise
#[derive(Deserialize, Debug, Clone, PartialEq, EnumString)]
pub enum Platform {
    #[serde(rename = "win")]
    #[strum(serialize = "win")]
    Win,
    #[serde(rename = "linux")]
    #[strum(serialize = "linux")]
    Linux,
    #[serde(rename = "macos-25.2")]
    #[strum(serialize = "macos-25.2")]
    MacOS252,

    #[serde(rename = "unknown")]
    #[strum(serialize = "unknown")]
    Unknown,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86 => write!(f, "x86"),
            Architecture::X64 => write!(f, "x86_64"),
            Architecture::ARM => write!(f, "arm"),
            Architecture::ARM64 => write!(f, "aarch64"),
            Architecture::RISCV64 => write!(f, "riscv64"),
        }
    }

}

impl Architecture{
    pub fn to_gcc_target_arch(&self) -> String {
        match self {
            Architecture::X86 => "i686".to_string(),
            Architecture::X64 => "x86_64".to_string(),
            Architecture::ARM => "arm".to_string(),
            Architecture::ARM64 => "aarch64".to_string(),
            Architecture::RISCV64 => "riscv64".to_string(),
        }
    }
}

impl Architecture {
    pub fn new(arch_str: &str) -> Result<Self, String> {
        Architecture::from_str(arch_str).map_err(|_| format!("Unsupported architecture: {}", arch_str))
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Win => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::MacOS252 => write!(f, "macos"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

impl Platform {
    pub fn new(platform_str: &str) -> Result<Self, String> {
        Platform::from_str(platform_str).map_err(|_| format!("Unsupported platform: {}", platform_str))
    }

    pub fn to_gcc_target_platform(&self) -> String {
        match self {
            Platform::Win => "w64-mingw32".to_string(),
            Platform::Linux => "linux-gnu".to_string(),
            Platform::MacOS252 => "apple-darwin-25.2".to_string(),
            Platform::Unknown => "unknown".to_string(),
        }
    }
}