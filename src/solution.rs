use std::path::PathBuf;
use serde::Deserialize;
use crate::target::{Architecture, Platform};

#[derive(Deserialize)]
pub struct Solution {
    pub name: String,
    pub projects: Vec<Project>,
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub enum ProjectType {
    StaticLib,
    DynamicLib,
    Executable,
}

// Implemented clone for Dependency to allow duplication when needed.
// TODO: Find a way to not use that if possible.
#[derive(Deserialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub project_type: ProjectType,
    pub target_archs: Vec<Architecture>,
    pub target_platforms: Vec<Platform>,
    pub path: PathBuf,
    pub dependencies: Vec<Dependency>,
    pub additional_includes: Vec<PathBuf>,
    #[serde(rename = "additional_static_libs", default)]
    pub additional_libs: Vec<PathBuf>,
}


impl Project {
    pub fn new(
        name: &str,
        version: &str,
        project_type: ProjectType,
        target_archs: Vec<Architecture>,
        target_platforms: Vec<Platform>,
        path: PathBuf,
        dependencies: Vec<Dependency>,
        additional_includes: Vec<PathBuf>,
        additional_libs: Vec<PathBuf>,
    ) -> Self {
        Project {
            name: name.to_string(),
            version: version.to_string(),
            project_type,
            target_archs,
            target_platforms,
            path,
            dependencies,
            additional_includes,
            additional_libs,
        }
    }
}