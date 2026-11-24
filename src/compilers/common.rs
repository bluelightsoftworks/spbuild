use std::path::{Path, PathBuf};

use crate::solution::{Project, Solution};

/// This is relatively useless as of right now but will be useful once more compilers are added
pub trait Compiler {
    ///
    /// Compiles a single source file.
    ///
    /// # Arguments
    /// * `abs_infile_path` - Absolute path to the input source file.
    /// * `abs_output_path` - Absolute path to the output file.
    /// * `additional_includes` - Additional include directories.
    /// # Returns
    /// * The path of the object file generated if success, otherwise an error message.
    ///
    fn compile_file(
        &self,
        abs_infile_path: &Path,
        abs_output_path: &Path,
        // Target platform and architecture are managed at compiler level
        project_path: &Path,
        additional_includes: &Vec<PathBuf>,
    ) -> Result<PathBuf, &'static str>;

    ///
    /// Compiles an entire project.
    ///
    /// # Arguments
    /// * `project` - The project to compile.
    /// * `solution` - The solution containing the project.
    /// * `solution_root` - The root path of the solution.
    /// * `additional_include_directories` - Additional include directories.
    /// # Returns
    /// * `Ok(())` if compilation is successful, otherwise an error message.
    /// # Note:
    /// This function checks for any circular dependencies in the project before proceeding with compilation.
    ///
    fn compile_project(
        &self,
        project: &Project,
        solution_root: &Path,
        additional_include_directories: Vec<PathBuf>,
    ) -> Result<(), &'static str>;

    ///
    /// Links the compiled object files of a project into a final executable or library.
    /// Right now, it only links together object files into an executable : libraries are still kept as object files.
    ///
    /// # Arguments
    /// * `project` - The project to link.
    /// * `solution` - The solution containing the project.
    /// * `project_path` - The path to the project.
    /// * `includes_paths` - Include paths for linking.
    /// # Returns
    /// * `Ok(())` if linking is successful, otherwise an error message.
    ///
    fn link_project(
        &self,
        project: &Project,
        solution: &Solution,
        project_path: &Path,
        includes_paths: Vec<PathBuf>,
    ) -> Result<(), &'static str>;
}
