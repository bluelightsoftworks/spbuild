use std::path::PathBuf;
use std::collections::{HashSet, VecDeque};
use serde::Deserialize;

use crate::enums::CppVersion;
use crate::helpers::console::Console;

use crate::structs::target::Target;
use crate::helpers::version_tools::version_check;

/// Lightweight dependency reference used in config files.
/// Only stores the information needed for matching/resolution.
#[derive(Deserialize, Clone, Eq, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub version: String,
}

#[derive(Deserialize, Clone)]
pub struct Solution {
    pub name: String,
    pub projects: Vec<Project>,

    #[serde(default)]
    pub compile_queue: VecDeque<Project>,
}

impl Solution {
    /// Tries to resolve a dependency as a local project listed in this solution file.
    pub fn find_local_dependency_match(&self, dependency: &Dependency) -> Option<Project> {
        self.projects
            .iter()
            .find(|p| p.name == dependency.name && version_check(&dependency.version, &p.version))
            .cloned()
    }

    /// Resolves a dependency reference against the local solution projects.
    ///
    /// Resolution order:
    /// 1) local solution project
    /// 2) online registry (placeholder for future implementation)
    pub fn resolve_project_reference(&self, dependency: &Dependency) -> Option<Project> {
        self.find_local_dependency_match(dependency)
    }

    /// Placeholder for future remote package resolution.
    fn fetch_project_from_registry(&self, _dependency: &Dependency) -> Option<Project> {
        todo!()
    }

    /// Builds a complete compile list from all projects in the solution.
    ///
    /// This recursively follows local dependencies and also appends each project's
    /// declared dependencies to `dependencies_to_compile`.
    pub fn create_compile_list(&mut self) {
        self.compile_queue.clear();

        let mut visited_projects: HashSet<String> = HashSet::new();

        for project in self.projects.clone() {
            self.collect_project_and_dependencies(&project, &mut visited_projects);
        }

        self.remove_duplicates();
    }

    fn collect_project_and_dependencies(
        &mut self,
        project: &Project,
        visited_projects: &mut HashSet<String>,
    ) {
        let project_key = format!("{}@{}", project.name, project.version);
        for dep in &project.dependencies {
            if let Some(resolved_dep) = self.resolve_project_reference(dep) {
                self.collect_project_and_dependencies(&resolved_dep, visited_projects);
            } else {
                Console::log_error(format!(
                    "Unresolved dependency: {} {}!",
                    dep.name, dep.version
                ).as_str());
                Console::log_error("==== Aborting build ====");
                std::process::exit(1);
            }
        }

        if visited_projects.insert(project_key) {
            self.compile_queue.push_back(project.clone());
        }
    }

    /// Removes duplicate projects/dependencies from the to compile list
    pub fn remove_duplicates(&mut self) {
        let mut seen: HashSet<String> = HashSet::new();
        self.compile_queue
            .retain(|p| seen.insert(format!("{}@{}", p.name, p.version)));
    }
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub enum ProjectType {
    StaticLibrary,
    DynamicLibrary,
    Executable,
}


/// High level view of a project. Doesn't store file paths because it is deserialized from the
/// config file
#[derive(Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub cpp_version: Option<CppVersion>,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub targets: Vec<Target>,
    pub path: PathBuf,
    pub exclude: Vec<PathBuf>,
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
        cpp_version: CppVersion,
        targets: Vec<Target>,
        path: PathBuf,
        exclude: Vec<PathBuf>,
        dependencies: Vec<Dependency>,
        additional_includes: Vec<PathBuf>,
        additional_libs: Vec<PathBuf>,
    ) -> Self {
        Project {
            name: name.to_string(),
            version: version.to_string(),
            project_type,
            cpp_version: Some(cpp_version),
            targets,
            path,
            exclude,
            dependencies,
            additional_includes,
            additional_libs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Solution;

    #[test]
    fn parses_lightweight_dependency_references() {
        let yaml = r#"
name: ExampleSolution
projects:
  - name: ProjectAlpha
    version: 1.0.0
    type: Executable
    cpp_version: 17
    targets:
      - x86_64-unknown-linux-gnu
    path: ./alpha/
    exclude: []
    dependencies:
      - name: ProjectBeta
        version: ^2.0.0
      - name: libraryB
        version: ~1.5.0
    additional_includes: []
    additional_static_libs: []
  - name: ProjectBeta
    version: 2.1.0
    type: StaticLibrary
    cpp_version: 20
    targets:
      - x86_64-unknown-linux-gnu
    path: ./beta/
    exclude: []
    dependencies: []
    additional_includes: []
    additional_static_libs: []
"#;

        let solution: Solution = serde_saphyr::from_str(yaml).expect("solution should parse");

        assert_eq!(solution.projects.len(), 2);
        assert_eq!(solution.projects[0].dependencies.len(), 2);
        assert_eq!(solution.projects[0].dependencies[0].name, "ProjectBeta");
        assert_eq!(solution.projects[0].dependencies[0].version, "^2.0.0");
    }

    #[test]
    fn creates_dependency_first_compile_order() {
        let yaml = r#"
name: ExampleSolution
projects:
  - name: ProjectAlpha
    version: 1.0.0
    type: Executable
    cpp_version: 17
    targets:
      - x86_64-unknown-linux-gnu
    path: ./alpha/
    exclude: []
    dependencies:
      - name: ProjectBeta
        version: ^2.0.0
    additional_includes: []
    additional_static_libs: []
  - name: ProjectBeta
    version: 2.1.0
    type: StaticLibrary
    cpp_version: 20
    targets:
      - x86_64-unknown-linux-gnu
    path: ./beta/
    exclude: []
    dependencies: []
    additional_includes: []
    additional_static_libs: []
"#;

        let mut solution: Solution = serde_saphyr::from_str(yaml).expect("solution should parse");
        solution.create_compile_list();

        assert_eq!(solution.compile_queue.len(), 2);
        assert_eq!(solution.compile_queue[0].name, "ProjectBeta");
        assert_eq!(solution.compile_queue[1].name, "ProjectAlpha");
    }
}
