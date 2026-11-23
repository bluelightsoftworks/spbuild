use crate::target::{Architecture, Platform};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Solution {
    pub name: String,
    pub projects: Vec<Project>,
}

impl Solution {
    pub fn new(name: &str, projects: Vec<Project>) -> Self {
        Solution {
            name: name.to_string(),
            projects,
        }
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone)]
pub enum ProjectType {
    StaticLib,
    DynamicLib,
    Executable,
    TestSuite,
}

impl fmt::Display for ProjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ProjectType::StaticLib => write!(f, "Static Library"),
            ProjectType::DynamicLib => write!(f, "Dynamic Library"),
            ProjectType::Executable => write!(f, "Executable"),
            ProjectType::TestSuite => write!(f, "Test Suite (not implemented yet)"),
        }
    }
}

// Implemented clone for Dependency to allow duplication when needed.
// TODO: Find a way to not use that if possible.
#[derive(Deserialize, Serialize, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

impl Dependency {
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

#[derive(Deserialize, Serialize, Clone)]
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

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
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

    pub fn to_dependency(&self) -> Dependency {
        Dependency::new(self.name.to_string(), self.version.to_string())
    }
}
