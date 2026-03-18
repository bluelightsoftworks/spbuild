use std::fs::{exists, create_dir_all};
use std::path::{Path, PathBuf};

use std::{env, io};
use std::io::Write;

use std::process::{Command, Stdio};
use crate::compiler_interfaces::common::Compiler;
use crate::helpers::console::Console;
use crate::helpers::file_tools::*;

use crate::solution::{Project, ProjectType, Solution};


pub struct GccCompiler {
    pub gcc_path: String,
    pub gpp_path: String,
    pub ld_path: String,

    pub is32bit: bool,
    pub t_arch: String,
    pub t_platform: String,
}

impl GccCompiler {

    pub fn new(t_arch: String, t_platform: String) -> Self {
        let target_spec = format!("{}-{}", t_arch, t_platform);
        let bin_path = Path::new("/usr/bin");
        let gcc_path = bin_path.join(format!("{target_spec}-gcc"));
        let gpp_path = bin_path.join(format!("{target_spec}-g++"));
        // MinGW ld path
        let ld_path: PathBuf = if t_platform == "w64-mingw32" {
            bin_path.join(format!("{}-ld", target_spec))
        } else {
            bin_path.join("ld")
        };

        let is32bit = t_arch == "i386" || t_arch == "i486" || t_arch == "i586" || t_arch == "i686" || t_arch == "x86";

        GccCompiler {
            gcc_path: gcc_path.to_str().unwrap_or("/usr/bin/gcc").to_string(),
            gpp_path: gpp_path.to_str().unwrap_or("/usr/bin/g++").to_string(),
            t_arch: t_arch.to_string(),
            t_platform: t_platform.to_string(),
            ld_path: ld_path.to_str().unwrap_or("/usr/bin/ld").to_string(),
            is32bit,
        }

    }


    ///
    /// Resolves DLL dependencies for a Windows executable by running the `cpdll.py` script.
    /// This script is expected to be located at `./scripts/cpdll.py` and should handle copying the necessary DLLs to the executable's directory.
    /// # Arguments
    /// * `exe_path` - The path to the executable for which to resolve DLL dependencies
    ///
    pub fn resolve_dlls(exe_path: &PathBuf, verbose: &bool) {
        let spbuild_root = match env::current_exe() {
            Ok(path) => path,
            Err(_) => {
                Console::log_warning("Unable to determine current executable path; skipping DLL resolution.");
                return;
            }
        };

        // Derive the path to the scripts directory from the spbuild executable path
        let cpdll_script_path = match spbuild_root.parent() {
            Some(parent) => parent.join("scripts").join("cpdll.py"),
            None => {
                Console::log_warning("Could not determine scripts directory; skipping DLL resolution.");
                return;
            }
        };

        if !cpdll_script_path.exists() {
            Console::log_warning("cp_dlls.py script not found. You will need to copy DLLs yourself");
            return;
        }

        Console::log_info("\n\n===> Running cpdll.py script for DLL dependency resolution...");

        // Command crafting: execute the script directly (honors shebang where supported)
        let mut command = Command::new(&cpdll_script_path);

        command.arg(exe_path);

        // Eventual logging configuration for the script execution
        if *verbose {
            command
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());

            Console::log_verbose(
                &format!(
                    "Executing command: {} {}",
                    &cpdll_script_path.display(),
                    exe_path.display()
                ),
                true,
            );
        }

        // Execution
        let output = command.output();

        // Error handling
        match output {
            Ok(output) => {
                if output.status.success() {
                    Console::log_info("DLLs copied successfully.");
                } else {
                    Console::log_error("cp_dlls.py script failed to execute. You will need to copy DLLs yourself");
                }
            }
            Err(_) => {
                Console::log_error("Failed to execute cp_dlls.py script. You will need to copy DLLs yourself");
            }
        }
    }

    ///
    /// Crafts the GCC command for compiling a single source file.
    ///
    /// # Arguments
    /// * `driver` - The path to the GCC driver (gcc or g++).
    /// * `abs_output_dir` - The absolute path to the output directory.
    /// * `abs_infile_path` - The absolute path to the input source file.
    /// * `abs_output_path` - The absolute path to the output object file.
    /// * `additional_includes` - A vector of additional include paths.
    /// # Returns
    /// A `Command` object representing the GCC command.
    ///
    fn craft_command(
        driver: &str,
        is32bit: bool,
        abs_output_dir: &PathBuf,
        abs_infile_path: &PathBuf,
        abs_output_path: &PathBuf,
        additional_includes: &Vec<PathBuf>) -> Command {

        // Crafts the command base
        let mut command = Command::new(driver);

        // Special 32 bit scenario
        if is32bit {
            command.arg("-m32");
        }

        command
            .current_dir(&abs_output_dir)
            .arg("-c")
            .arg(&abs_infile_path)
            .arg("-o")
            .arg(&abs_output_path);

        // Adds includes
        for include_path in additional_includes {
            // Include paths are expected to be absolute or already correctly rooted.
            command.arg("-I").arg(include_path);
        }

        command
    }

}

impl Compiler for GccCompiler {

    fn compile_file(
        &self,
        abs_infile_path: &PathBuf,
        abs_output_dir: &PathBuf,
        additional_includes: &Vec<PathBuf>,
        _verbose: bool,
    ) -> Result<(), &'static str> {

        // Checks for edge cases I don't even know if they can happen
        let last_component_abs_output_dir = abs_infile_path.components().last();
        if last_component_abs_output_dir.is_none() {
            return Err("Invalid input file path");
        }

        let abs_output_path = abs_output_dir.join(last_component_abs_output_dir.unwrap()).with_added_extension("o");

        // Check extension
        if abs_infile_path.extension() != Some(std::ffi::OsStr::new("c")) &&
           abs_infile_path.extension() != Some(std::ffi::OsStr::new("cpp")) &&
           abs_infile_path.extension() != Some(std::ffi::OsStr::new("cc")) &&
           abs_infile_path.extension() != Some(std::ffi::OsStr::new("cxx")) {
            Console::log_warning(format!("Unsupported source file extension for GCC compiler: {}", &abs_infile_path.display()).as_str());
            return Ok(());
        }

        if let Some(parent) = abs_output_path.parent() {
            create_dir_all(parent).map_err(|_| "Failed to create output subdirectory")?;
        }

        // Use g++ for C++ sources so the preprocessor selects the right language.
        let driver = if abs_infile_path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("cpp") || e.eq_ignore_ascii_case("cc") || e.eq_ignore_ascii_case("cxx"))
        {
            &self.gpp_path
        } else {
            &self.gcc_path
        };

        Console::log_verbose(&format!("input:  {}", abs_infile_path.display()), _verbose);
        Console::log_verbose(&format!("output: {}", abs_output_path.display()), _verbose);

        let mut command = Self::craft_command(&driver, self.is32bit, &abs_output_dir, &abs_infile_path, &abs_output_path, &additional_includes);

        // Executes the command
        let output = command
            .output()
            .map_err(|_| "Failed to execute GCC")?;

        Console::log_info(&format!("status: {}", output.status));

        io::stdout().write_all(&output.stdout).map_err(|_| "Failed to write to stdout")?;
        io::stderr().write_all(&output.stderr).map_err(|_| "Failed to write to stderr")?;

        if output.status.success() {
            Console::log_info(&format!(">> {} compiled successfully!", abs_infile_path.display()).as_str());
            Ok(())
        } else {
            Err("Compilation failed.")
        }
    }


    fn compile_project(
        &self,
        project: &Project,
        solution_root: &PathBuf,
        include_directories: Vec<PathBuf>,
        _verbose: bool,
    ) -> Result<(), &'static str> {

        let gpp_path = &self.gpp_path;
        let abs_solution_root = solution_root.canonicalize().map_err(|_| "Failed to canonicalize solution root path")?;

        Console::log_info(&format!(
            "Compiling Project: {} version {} ({}) using G++ at {}\n",
            project.name,
            project.version,
            project.path.display(),
            &gpp_path
        ));

        let source_dir = solution_root
            .join(&project.path)
            .canonicalize()
            .map_err(|e| {
                Console::log_error(&format!("Project source directory not found: {} - {}", project.path.display(), e));
                "Project source directory not found"
            })?;

        let files = list_files(&source_dir).map_err(|_| "Failed to list source files")?;

        let rel_output_dir = &abs_solution_root
            .join("output")
            .join(format!("{}-{}", &self.t_platform, &self.t_arch))
            .join(&project.path);

        if !exists(rel_output_dir).unwrap_or(false) {
            // Creates output directory if it doesn't exist
            Console::log_verbose(&format!("Project output directory not found: {}", project.path.display()), _verbose);
            create_dir_all(rel_output_dir).map_err(|_| "Failed to create output directory")?;
        };

        let abs_output_dir = rel_output_dir.canonicalize().map_err(|_| "Failed to canonicalize output directory")?;

        for source_file in files {
            // list_files returns paths like ./main.c relative to source_dir
            let rel = source_file.to_string_lossy().into_owned();
            let project_path_str = &project.path.to_string_lossy().into_owned();

            let abs_source_file = solution_root
                .join(project_path_str)
                .join(&source_file)
                .canonicalize()
                .map_err(|_| "Failed to canonicalize path. The file likely doesn't exist")?;

            Console::log_info(&format!("Compiling source file: {}", &rel));
            self.compile_file(
                &abs_source_file,
                &abs_output_dir,
                &include_directories, _verbose)?;
        }

        Ok(())
    }


    fn link_project(
        &self, project: &Project,
        _solution: &Solution, // Will probably be used
        solution_root: &PathBuf,
        includes_paths: Vec<PathBuf>,
        verbose: bool) -> Result<(), &'static str> {

        if project.project_type == ProjectType::StaticLib
        {
            Console::log_info("Static library project detected; skipping linking step.");
            return Ok(());
        }

        // Absolute path to the project's output directory containing object files.
        let abs_project_output_path = &solution_root
            .join("output")
            .join(format!("{}-{}", &self.t_platform, &self.t_arch))
            .join(&project.path)
            .canonicalize()
            .map_err(|_| {"Project Output Path not found"})?;
        let files = list_files(&abs_project_output_path).map_err(|_| "Failed to list object files")?;

        // Project's object files
        let mut object_files = find_object_files(&files, &abs_project_output_path);

        if object_files.is_empty() {
            Console::log_warning(format!("No object files were found in directory: {}. This may be unintended behavior", abs_project_output_path.display()).as_str());
            return Ok(());  // Nothing to link, but not an error.
        }

        // Dependencies' object files
        for path in includes_paths {
            let files = list_files(&path).map_err(|_| "Failed to list dependency files")?;
            let mut dep_object_files = find_object_files(&files, &path);
            object_files.append(&mut dep_object_files);
        }

        // Additional static libraries
        for lib_path in &project.additional_libs {
            // Resolve library paths relative to the project root so existence checks
            // and linking do not depend on the current working directory.
            let resolved_lib_path = {
                let base_path = if lib_path.is_relative() {
                    solution_root.join(&project.path).join(lib_path)
                } else {
                    lib_path.clone()
                };
                // Canonicalize when possible, but fall back to the joined path if it fails.
                base_path.canonicalize().unwrap_or(base_path)
            };

            if exists(&resolved_lib_path).unwrap_or(false) {
                Console::log_verbose(
                    &format!("Adding additional library to link: {}", resolved_lib_path.display()),
                    verbose,
                );
                object_files.push(resolved_lib_path);
            } else {
                Console::log_error(&format!(
                    "Additional library not found: {}.",
                    resolved_lib_path.display()
                ));
                Err("Failed to find additional library for linking")?;
            }
        }

        let mut command = Command::new(&self.gpp_path);

        // For project `alpha`, output executable is at `<project_root>/output/alpha/alpha`.
        let output_executable = abs_project_output_path.join(&project.name);


        Console::log_info(&format!("Linking executable: {}", output_executable.display()));

        // Adds object files to the linker command
        command.current_dir(&abs_project_output_path);
        for obj in &object_files {
            command.arg(obj);
        }

        // Specific for Windows targets (DLLs and exe size limitations)
        if &self.t_platform == "w64-mingw32" {
            let exe_with_ext = output_executable.with_added_extension("exe");
            Console::log_verbose(&format!("Windows target detected; using executable name: {}", exe_with_ext.display()), verbose);

            // TODO: Give the user the option to choose static or dynamic linking for Windows targets.
            // Also use DLLs when available
            command
                //.arg("--static")
                .arg("-o").arg(&exe_with_ext);
        }
        else {
            command.arg("-o").arg(&output_executable);
        }

        Console::log_verbose(&format!("Linking command: {:?}", command), verbose);

        let output = command
            .output()
            .map_err(|_| "Failed to execute GCC for linking")?;

        println!("status: {}", output.status);

        io::stdout().write_all(&output.stdout).map_err(|_| "Failed to write to stdout")?;
        io::stderr().write_all(&output.stderr).map_err(|_| "Failed to write to stderr")?;
        if output.status.success() {
            Console::log_success("Linked successfully.");
            Ok(())
        } else {
            Err("Linking failed.")
        }

    }
}