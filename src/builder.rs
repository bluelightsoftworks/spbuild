use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde_json::Value;
use crate::linker::Linker;
use crate::structs::target::{Architecture, Platform};

use crate::enums::CppVersion;
use crate::helpers::file_tools::create_directory_if_not_exists;
use crate::structs::solution::ProjectType;

pub struct Builder {
    additional_include_paths: Vec<PathBuf>,
    files: Vec<PathBuf>,
    defines: HashMap<String, String>,
    dependencies_compiled_files: Vec<PathBuf>,

    cpp_version: CppVersion,
    warning_level: usize, // 0, (1), 2 or 3 (nothing, standard warnings, extra warnings or warnings as errors)
    flags: Vec<String>,
    env_vars: HashMap<String, String>,

    target_str: String,
    output_files_paths: Vec<PathBuf>,

    host_triplet: String,
    project_type: ProjectType,

    // Linker and compiler objects attached to the builder
    compiler: cc::Build,
    linker: Linker
}

impl Builder {

    pub fn new(target: &str, host_triplet: &String, project_type: &ProjectType) -> Self {
        Builder
        {
            additional_include_paths: vec![],
            files: vec![],
            dependencies_compiled_files: vec![],
            defines: HashMap::new(),
            cpp_version: CppVersion::CPP20, // Assumes newest version if not specified
            warning_level: 1, // Default
            flags: vec![],
            env_vars: HashMap::new(),
            target_str: target.to_string(),
            output_files_paths: vec![],
            host_triplet: host_triplet.clone(),
            project_type: project_type.clone(),

            compiler: cc::Build::new(),
            linker: Linker::new(target).unwrap_or_else(|_| panic!("No supported linker found for target: {}", target)),
        }
    }

    /// Adds a new file to the compile list
    pub fn add_file(&mut self, file_path: &Path)
    {
        self.files.push(file_path.to_path_buf());
    }

    /// Adds a new file to the list of compiled files from dependencies
    pub fn add_dependency_compiled_file(&mut self, file_path: &Path)
    {
        self.dependencies_compiled_files.push(file_path.to_path_buf());
    }

    /// Adds a new directory to include from
    pub fn add_include(&mut self, path: &Path){
        self.additional_include_paths.push(path.to_path_buf());
    }

    /// Sets the version of the std lib that should be used for compilation
    pub fn set_cpp_version(&mut self, cpp_version: &CppVersion) {
        self.cpp_version = cpp_version.clone();
    }

    /// Sets the version of the standard library (libc++, libstdc++, libstdc++_shared, etc...).
    /// Defaults should be libc++ for targets MacOS and Free/OpenBSD and libstdc++ for everything else
    /// Setting this is optional
    pub fn set_stdlib(&mut self, stdlib: String) {
        self.compiler.cpp_link_stdlib(&*stdlib);
    }

    /// Sets the warning level:
    /// - 0: disabled
    /// - 1: normal warnings
    /// - 2: extra warnings
    /// - 3: warnings treated as errors
    pub fn set_warning_level(&mut self, level: usize) {
        self.warning_level = level;
    }

    /// Sets a flag for compilation
    pub fn set_flag(&mut self, flag: String, value: String) {

        // check if the Hashmap already got that flag set
        self.flags.push(flag);
    }

    /// Sets a define value for the preprocessor
    pub fn add_define(&mut self, define: String, value: String) {
        // check if the Hashmap already got that define set
        if self.defines.contains_key(&define) {
            self.defines.remove(&define);
            self.defines.insert(define, value);
        }
        else {
            self.defines.insert(define, value);
        }
    }

    /// Sets an environment variable for the compiler
    pub fn set_env_var(&mut self, var: &str, value: &str) {
        // check if the Hashmap already got that env var set
        if self.env_vars.contains_key(var) {
            self.env_vars.remove(var);
            self.env_vars.insert(var.to_string(), value.to_string());
        }
        else {
            self.env_vars.insert(var.to_string(), value.to_string());
        }
    }

    /// Sets the target for compilation (e.g. "x86_64-pc-windows-msvc" or "aarch64-apple-darwin")
    pub fn set_target_str(&mut self, target_str: String) {
        self.target_str = target_str;
    }

    /// Compiles files into intermediary files. Doesn't link them
    pub fn build(&mut self, output_path: &Path) -> Result<(), String> {

        let outdir = output_path;
        create_directory_if_not_exists(&outdir).unwrap_or_else(|_| panic!("Failed to create output directory: {}", &outdir.display()));

        self.compiler
            .cpp(true)
            .target(&self.target_str)
            .opt_level(0)
            .host(&self.host_triplet)
            .flags(&self.flags)
            .files(&self.files)
            .includes(&self.additional_include_paths)
            .out_dir(outdir.canonicalize().unwrap());

        // adds to the list of compiled files for the linker
        for file in &self.files {
            let name = file.file_prefix().unwrap().to_str().unwrap().to_string() + ".o";
            self.output_files_paths.push(output_path.join(name));
        }

        // Sets every environment variables
        for var in &self.env_vars {
            self.compiler.env(var.0, var.1);
        }

        // Sets every define for the preprocessor
        for define in &self.defines {
            self.compiler.define(define.0, Some(define.1.as_str()));
        }

        match self.warning_level {
            0 => {
                self.compiler.warnings(false);
            },
            1 => {
                self.compiler.warnings(true);
            },
            2 => {
                self.compiler.extra_warnings(true);
            },
            3 => {
                self.compiler.warnings(true);
                self.compiler.extra_warnings(true);
                self.compiler.warnings_into_errors(true);
            },
            _ => {
                panic!("Invalid warning level: {}", self.warning_level);
            }
        }

        match self.cpp_version {
            CppVersion::CPP20 => {
                self.compiler.cpp(true);
                self.compiler.std("c++20");
            },
            CppVersion::CPP17 => {
                self.compiler.cpp(true);
                self.compiler.std("c++17");
            },
            CppVersion::CPP14 => {
                self.compiler.cpp(true);
                self.compiler.std("c++14");
            }
        }

        // compiles the thing into a single file
        let compiled_result = self.compiler.compile_intermediates();

        if self.project_type == ProjectType::Executable {
            // links everything if target is executable
            self.linker.link(&self.target_str, compiled_result);
        }

        Ok(())
    }
}