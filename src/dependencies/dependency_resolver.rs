use crate::dependencies::external_resolver::resolve_additional_include_directories;
use crate::dependencies::local_resolver::resolve_local_deps;
use crate::helpers::console::Console;
use crate::solution::{Project, Solution};
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

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
    /// For a dependencies at `<solution_root>/<dep.path>`, this is typically:
    /// `<solution_root>/output/<project.target>/<dep.path>`.
    pub dep_output_dirs: Vec<PathBuf>,
}

/// Computes include directories for `project`.
///
/// - Validates that `project.additional_includes` exist relative to `<solution_root>/<project.path>`.
/// - Adds each local dependencies' project root as an include directory.
///
/// # Returns
/// A ProjectBuildInputs struct
pub fn resolve_dependencies(
    project: &Project,
    solution: &Solution,
    solution_root: &Path,
    target_string: &String,
) -> Result<ProjectBuildInputs, String> {
    let local_deps_in_order = resolve_local_deps(project, solution);

    let mut include_dirs: Vec<PathBuf> = Vec::new();
    let mut dep_output_dirs: Vec<PathBuf> = Vec::new();

    // Project additional include dirs
    let additional_includes = resolve_additional_include_directories(
        &project.additional_includes,
        &solution_root.join(&project.path),
    )?;
    include_dirs.extend(additional_includes);

    // Local dependencies roots as include dirs + dependencies output dirs for linking
    for dep in &local_deps_in_order {
        let dep_root = solution_root.join(&dep.path).canonicalize();

        if dep_root.is_err() {
            Console::log_fatal("Failed to locate dependencies source directory");
        }
        include_dirs.push(dep_root.unwrap());

        // Dependency objects are placed in `<solution_root>/output/<project.target>/<dep.path>` by the compiler backend.
        // Canonicalize so link inputs are absolute.
        let abs_dep_output_dir = solution_root
            .join("build")
            .join(target_string)
            .join(&dep.path)
            .canonicalize();

        if abs_dep_output_dir.is_err() {
            create_dir_all(
                solution_root
                    .join("build")
                    .join(target_string)
                    .join(&dep.path),
            )
            .map_err(|_| "Failed to create dependencies output directory")?;
        }

        // checks if there is still an error after creating the directories
        let abs_dep_output_dir = solution_root
            .join("build")
            .join(target_string)
            .join(&dep.path)
            .canonicalize()
            .map_err(|_| "Failed to create dependencies output directory")?;

        dep_output_dirs.push(abs_dep_output_dir);
    }

    Ok(ProjectBuildInputs {
        local_deps_in_order,
        include_dirs,
        dep_output_dirs,
    })
}
