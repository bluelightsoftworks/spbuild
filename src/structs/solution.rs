use std::path::PathBuf;
use std::collections::HashSet;
use serde::Deserialize;
use crate::structs::target::Target;
use crate::helpers::version_tools::version_check;

#[derive(Deserialize, Clone)]
pub struct Solution {
    pub name: String,
    pub projects: Vec<Project>,
    #[serde(default)]
    pub projects_to_compile: Vec<Project>,
    #[serde(default)]
    pub dependencies_to_compile: Vec<Project>,
}

impl Solution {
    /// Tries to resolve a dependency as a local project listed in this solution file.
    pub fn find_local_project_match(&self, dependency: &Project) -> Option<Project> {
        self.projects
            .iter()
            .find(|p| p.name == dependency.name && version_check(&dependency.version, &p.version))
            .cloned()
    }

    /// Resolves a dependency reference into a full project object.
    ///
    /// Resolution order:
    /// 1) local solution project
    /// 2) online registry (placeholder for future implementation)
    pub fn resolve_project_reference(&self, dependency: &Project) -> Option<Project> {
        self.find_local_project_match(dependency)
            .or_else(|| self.fetch_project_from_registry(dependency))
    }

    /// Placeholder for future remote package resolution.
    fn fetch_project_from_registry(&self, _dependency: &Project) -> Option<Project> {
        todo!()
    }

    /// Builds a complete compile list from all projects in the solution.
    ///
    /// This recursively follows local dependencies and also appends each project's
    /// declared dependencies to `dependencies_to_compile`.
    pub fn build_compile_list(&mut self) {
        self.projects_to_compile.clear();
        self.dependencies_to_compile.clear();

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
        if visited_projects.insert(project_key) {
            self.projects_to_compile.push(project.clone());
        }

        for dep in &project.dependencies {
            if let Some(resolved_dep) = self.resolve_project_reference(dep) {
                // Keep resolved dependencies for compile planning.
                self.dependencies_to_compile.push(resolved_dep.clone());
                self.collect_project_and_dependencies(&resolved_dep, visited_projects);
            } else {
                // Keep unresolved references so later phases can decide (e.g., registry pull).
                self.dependencies_to_compile.push(dep.clone());
            }
        }
    }

    // Removes duplicate projects/dependencies from the to compile list
    pub fn remove_duplicates(&mut self){
        let mut seen_projects: HashSet<String> = HashSet::new();
        self.projects_to_compile
            .retain(|p| seen_projects.insert(format!("{}@{}", p.name, p.version)));

        let mut seen_deps: HashSet<String> = HashSet::new();
        self.dependencies_to_compile
            .retain(|d| seen_deps.insert(format!("{}@{}", d.name, d.version)));
    }
}

#[derive(Deserialize, Eq, PartialEq, Clone)]
pub enum ProjectType {
    StaticLib,
    DynamicLib,
    Executable,
}

/// High level view of a project. Doesn't store file paths because it is deserialized from the
/// config file
#[derive(Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub project_type: ProjectType,
    pub targets: Vec<Target>,
    pub path: PathBuf,
    pub dependencies: Vec<Project>,
    pub additional_includes: Vec<PathBuf>,
    #[serde(rename = "additional_static_libs", default)]
    pub additional_libs: Vec<PathBuf>,
}


impl Project {
    pub fn new(
        name: &str,
        version: &str,
        project_type: ProjectType,
        targets: Vec<Target>,
        path: PathBuf,
        dependencies: Vec<Project>,
        additional_includes: Vec<PathBuf>,
        additional_libs: Vec<PathBuf>,
    ) -> Self {
        Project {
            name: name.to_string(),
            version: version.to_string(),
            project_type,
            targets,
            path,
            dependencies,
            additional_includes,
            additional_libs,
        }
    }
}