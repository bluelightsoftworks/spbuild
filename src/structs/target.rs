use std::str::FromStr;
use std::fmt;
use std::env;
use serde::Deserialize;
use serde::de::Deserializer;
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


//TODO: These may not be the right platforms.. Maybe we'll have to make this more precise
#[derive(Deserialize, Debug, Clone, PartialEq, EnumString)]
pub enum Platform {
    #[serde(rename = "win")]
    #[serde(alias = "windows")]
    #[strum(serialize = "win")]
    #[strum(serialize = "windows")]
    Win,
    #[serde(rename = "linux")]
    #[strum(serialize = "linux")]
    Linux,
    #[serde(rename = "macos-25.2")]
    #[serde(alias = "macos")]
    #[strum(serialize = "macos-25.2")]
    #[strum(serialize = "macos")]
    MacOS252,

    #[serde(rename = "unknown")]
    #[strum(serialize = "unknown")]
    Unknown,
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


#[derive(Clone)]
pub struct Target{
    pub architecture: Architecture,
    pub platform: Platform,
}

impl Target {
    pub fn to_triple(&self) -> String {
        match self.platform {
            Platform::Win => format!("{}-pc-windows-msvc", self.architecture),
            Platform::Linux => format!("{}-unknown-linux-gnu", self.architecture),
            Platform::MacOS252 => format!("{}-apple-darwin", self.architecture),
            Platform::Unknown => format!("{}-unknown-unknown", self.architecture),
        }
    }

    pub fn current_host_triple() -> Result<String, String> {
        let current_platform = env::consts::OS;
        let current_architecture = env::consts::ARCH;

        let normalized_platform = match current_platform {
            "windows" => "win",
            other => other,
        };

        let platform = Platform::new(normalized_platform)?;
        let architecture = Architecture::new(current_architecture)?;

        Ok(Target {
            architecture,
            platform,
        }
        .to_triple())
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TargetInput {
    Triple(String),
    Expanded {
        architecture: Architecture,
        platform: Platform,
    },
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = TargetInput::deserialize(deserializer)?;

        match input {
            TargetInput::Expanded {
                architecture,
                platform,
            } => Ok(Target {
                architecture,
                platform,
            }),
            TargetInput::Triple(target) => {
                // Expected format is similar to: <arch>-<vendor>-<os>-<abi>
                let parts: Vec<&str> = target.split('-').collect();
                if parts.len() < 3 {
                    return Err(serde::de::Error::custom(format!(
                        "Invalid target string '{}'. Expected at least 3 segments.",
                        target
                    )));
                }

                let architecture = Architecture::new(parts[0])
                    .map_err(serde::de::Error::custom)?;

                let os_token = parts
                    .iter()
                    .find(|segment| {
                        matches!(
                            **segment,
                            "windows" | "win" | "linux" | "macos" | "darwin"
                        )
                    })
                    .copied()
                    .ok_or_else(|| {
                        serde::de::Error::custom(format!(
                            "Could not determine platform from target string '{}'",
                            target
                        ))
                    })?;

                let normalized_platform = match os_token {
                    "darwin" => "macos",
                    other => other,
                };

                let platform = Platform::new(normalized_platform)
                    .map_err(serde::de::Error::custom)?;

                Ok(Target {
                    architecture,
                    platform,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Architecture, Platform, Target};

    #[test]
    fn deserialize_target_from_triple_string() {
        let target: Target = serde_json::from_str("\"x86_64-unknown-linux-gnu\"")
            .expect("target triple should deserialize");

        assert_eq!(target.architecture, Architecture::X64);
        assert_eq!(target.platform, Platform::Linux);
    }

    #[test]
    fn deserialize_target_from_expanded_object() {
        let target: Target = serde_json::from_str(
            r#"{"architecture":"x86_64","platform":"windows"}"#,
        )
        .expect("expanded target should deserialize");

        assert_eq!(target.architecture, Architecture::X64);
        assert_eq!(target.platform, Platform::Win);
    }

    #[test]
    fn target_to_triple_uses_platform_specific_format() {
        let linux = Target {
            architecture: Architecture::X64,
            platform: Platform::Linux,
        };

        assert_eq!(linux.to_triple(), "x86_64-unknown-linux-gnu");

        let windows = Target {
            architecture: Architecture::X64,
            platform: Platform::Win,
        };

        assert_eq!(windows.to_triple(), "x86_64-pc-windows-msvc");
    }
}
