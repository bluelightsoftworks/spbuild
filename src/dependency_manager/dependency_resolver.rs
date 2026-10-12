use std::fs::create_dir_all;
use std::path::PathBuf;
use crate::structs::solution::{Project, Solution};
use crate::Console;
use crate::helpers::version_tools::version_check;

///
/// Finds direct local dependencies of a given project within the solution.
/// # Arguments
/// * `project` - The project whose dependencies are to be resolved.
/// * `solution` - The solution containing all projects.
/// # Returns
/// * A `Vec<Project>` containing the resolved local dependencies.
///
pub fn find_direct_local_dependencies(
    project: &Project,
    solution: &Solution,
    verbose: bool,
) -> Vec<Project> {

    let mut found_deps: Vec<Project> = Vec::new();

    for dep in &project.dependencies {
        if let Some(local_dep) = solution.find_local_dependency_match(dep) {
            if project.name == local_dep.name {
                continue;
            }

            Console::log_verbose(
                format!(
                    "Checking dependency {} version {} => local match {} version {}",
                    dep.name, dep.version, local_dep.name, local_dep.version
                )
                .as_str(),
                verbose,
            );
            Console::log_verbose(
                format!(
                    "Result of version check: {}",
                    version_check(&dep.version, &local_dep.version)
                )
                .as_str(),
                verbose,
            );

            Console::log_info(
                format!(
                    "Found local dependency: {} version {} at path {}",
                    local_dep.name,
                    local_dep.version,
                    local_dep.path.display()
                )
                .as_str(),
            );

            found_deps.push(local_dep);
        }
    }

    found_deps
}

///
/// Checks for circular dependencies in the project graph.
/// Uses Recursion to traverse dependencies.
/// # Arguments
/// * `project` - The project to check for circular dependencies.
/// * `solution` - The solution containing all projects.
/// * `recursion_stack` - A mutable vector to keep track of the active recursion path during the check.
/// # Returns
/// * A boolean indicating whether a circular dependency exists.
///
pub fn has_circular_dependency(
    project: &Project,
    solution: &Solution,
    recursion_stack: &mut Vec<String>,
) -> bool {
    let project_key = format!("{}@{}", project.name, project.version);

    if recursion_stack.contains(&project_key) {
        return true; // Circular dependency detected
    }

    recursion_stack.push(project_key);

    for dep in &project.dependencies {
        if let Some(next) = solution.find_local_dependency_match(dep)
            && has_circular_dependency(&next, solution, recursion_stack) {
                return true;
            }
    }

    recursion_stack.pop();
    false
}

pub fn collect_header_files(folder: PathBuf) -> Vec<PathBuf> {

    let mut header_files: Vec<PathBuf> = Vec::new();

    let files = crate::helpers::file_tools::list_files(&folder);

    match files {
        Ok(file_list) => {
            for file in file_list {
                if let Some(ext) = file.extension() {
                    if ext == "h" || ext == "hpp" || ext == "hh" {
                        header_files.push(folder.join(&file));
                    }
                }
            }
        },
        Err(e) => {
            Console::log_error(format!("Error listing files in folder {}: {}", folder.display(), e).as_str());
        }
    }

    header_files
}

/// Build inputs computed for a single project.
#[derive(Clone)]
pub struct ProjectBuildInputs {
    /// Local projects that must be built before `project`.
    /// This is transitive and ordered so dependencies appear before dependents.
    pub local_deps_in_order: Vec<Project>,
    /// Include directories to pass to the compiler when compiling `project`.
    /// These are absolute or solution-root-relative paths (caller decides how to interpret).
    pub include_dirs: Vec<PathBuf>,
    /// Directories that contain linkable outputs for local deps (currently: `.o` files).
    ///
    /// For a dependency at `<solution_root>/<dep.path>`, this is typically:
    /// `<solution_root>/output/<target_triple>/<dep.path>`.
    pub dep_output_dirs: Vec<PathBuf>,
}

/// Resolves local (in-solution) dependencies in a dependency-first order.
///
/// Notes:
/// - This is transitive: if A depends on B and B depends on C, the result for A is [C, B].
/// - Missing local deps are ignored here (they may be external/system deps).
pub fn resolve_local_dependencies_in_order(
    project: &Project,
    solution: &Solution,
) -> Vec<Project> {
    fn dfs_visit(
        current: &Project,
        solution: &Solution,
        resolved: &mut Vec<Project>,
        active_path: &mut Vec<String>,
    ) {
        if active_path.contains(&current.name) {
            // Circular dependency should be checked earlier via has_circular_dependency.
            return;
        }
        active_path.push(current.name.clone());

        for dep in &current.dependencies {
            if let Some(resolved_dep) = solution.find_local_dependency_match(dep) {
                dfs_visit(&resolved_dep, solution, resolved, active_path);

                if !resolved.iter().any(|p| p.name == resolved_dep.name && p.version == resolved_dep.version) {
                    resolved.push(resolved_dep);
                }
            }
        }

        active_path.pop();
    }

    let mut resolved = Vec::new();
    let mut active_path = Vec::new();
    dfs_visit(project, solution, &mut resolved, &mut active_path);
    resolved
}

/// Computes include directories for `project`.
///
/// - Validates that `project.additional_includes` exist relative to `<solution_root>/<project.path>`.
/// - Adds each local dependency's project root as an include directory.
pub fn resolve_project_build_inputs(
    project: &Project,
    solution: &Solution,
    solution_root: &PathBuf,
    target_triple: &str,
    verbose: bool,
) -> Result<ProjectBuildInputs, &'static str> {
    let local_deps_in_order = resolve_local_dependencies_in_order(project, solution);

    let mut include_dirs: Vec<PathBuf> = Vec::new();
    let mut dep_output_dirs: Vec<PathBuf> = Vec::new();

    // Project additional include dirs
    for inc in &project.additional_includes {
        let abs_inc_path = solution_root.join(&project.path).join(inc);
        Console::log_verbose(
            format!("Adding additional include path: {}", abs_inc_path.display()).as_str(),
            verbose,
        );

        if !abs_inc_path.exists() {
            Console::log_error(
                format!("Additional include path does not exist: {}", abs_inc_path.display()).as_str(),
            );
            return Err("Failed to add additional include path");
        }

        include_dirs.push(abs_inc_path);
    }

    // Local dependency roots as include dirs + dependency output dirs for linking.
    for dep in &local_deps_in_order {
        let dep_root = solution_root
            .join(&dep.path)
            .canonicalize();


        if dep_root.is_err(){
            Console::log_error("Failed to locate dependency source directory");
        }
        include_dirs.push(dep_root.unwrap());

        // Dependency objects are placed in `<solution_root>/output/<target_triple>/<dep.path>` by the compiler backend.
        // Canonicalize so link inputs are absolute.
        let abs_dep_output_dir = solution_root
            .join("output")
            .join(target_triple)
            .join(&dep.path)
            .canonicalize();

        if abs_dep_output_dir.is_err() {
            create_dir_all(
                solution_root
                    .join("output")
                    .join(target_triple)
                    .join(&dep.path)
            )
                .map_err(|_| "Failed to create dependency output directory")?;
        }

        // checks if there is still an error after creating the directories
        let abs_dep_output_dir = solution_root
            .join("output")
            .join(target_triple)
            .join(&dep.path)
            .canonicalize()
            .map_err(|_| "Failed to create dependency output directory")?;

        dep_output_dirs.push(abs_dep_output_dir);
    }

    Ok(ProjectBuildInputs {
        local_deps_in_order,
        include_dirs,
        dep_output_dirs,
    })
}
