use std::path::{Path, PathBuf};
use std::collections::HashMap;

use cc;
use crate::linker::Linker;
use crate::structs::target::{Architecture, Platform};

pub enum StdVersion {
    CPP14,
    CPP17,
    CPP20,
}

pub struct Builder {
    additional_include_paths: Vec<PathBuf>,
    files: Vec<PathBuf>,
    defines: HashMap<String, String>,

    std_version: StdVersion,
    warning_level: usize, // 0, (1), 2 or 3 (nothing, standard warnings, extra warnings or warnings as errors)
    flags: Vec<String>,
    env_vars: HashMap<String, String>,

    target_str: String,
    output_files_paths: Vec<PathBuf>,

    host_arch: Architecture,
    host_platform: Platform,

    // Linker and compiler objects attached to the builder
    compiler: cc::Build,
    linker: Linker
}

impl Builder {

    pub fn new(target: &str, host_arch: Architecture, host_platform: Platform) -> Self {
        Builder
        {
            additional_include_paths: vec![],
            files: vec![],
            defines: HashMap::new(),
            std_version: StdVersion::CPP20, // Assumes newest version if not specified
            warning_level: 1, // Default
            flags: vec![],
            env_vars: HashMap::new(),
            target_str: target.to_string(),
            output_files_paths: vec![],
            host_arch,
            host_platform,

            compiler: cc::Build::new(),
            linker: Linker::new(&target)
        }
    }

    /// Adds a new file to the compile list
    pub fn add_file(&mut self, file_path: &Path)
    {
        self.files.push(file_path.to_path_buf());
    }

    /// Adds a new directory to include from
    pub fn add_include(&mut self, path: &Path){
        self.additional_include_paths.push(path.to_path_buf());
    }

    /// Sets the version of the std lib that should be used for compilation
    pub fn set_std_version(&mut self, std_version: StdVersion) {
        self.std_version = std_version;
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
    pub fn set_define(&mut self, define: String, value: String) {
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
    pub fn set_env_var(&mut self, var: String, value: String) {
        // check if the Hashmap already got that env var set
        if self.env_vars.contains_key(&var) {
            self.env_vars.remove(&var);
            self.env_vars.insert(var, value);
        }
        else {
            self.env_vars.insert(var, value);
        }
    }

    /// Sets the target for compilation (e.g. "x86_64-pc-windows-msvc" or "aarch64-apple-darwin")
    pub fn set_target_str(&mut self, target_str: String) {
        self.target_str = target_str;
    }

    /// Compiles files into intermediary files. Doesn't link them
    pub fn compile(&mut self, output_path: &Path) {

        self.compiler
            .target(&self.target_str)
            .flags(&self.flags)
            .files(&self.files)
            .includes(&self.additional_include_paths)
            .out_dir(output_path.canonicalize().unwrap());

        // adds to the list of compiled files for the linker
        for file in &self.files {
            let name = file.file_name().unwrap().to_str().unwrap().to_string();
            self.output_files_paths.push(output_path.join(name));
        }

        // Sets every environment variables
        for var in &self.env_vars {
            self.compiler.env(var.0, var.1);
        }

        // Sets every define for the preprocessor
        for define in &self.defines {
            self.compiler.define(&define.0, Some(define.1.as_str()));
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

        match self.std_version {
            StdVersion::CPP20 => {
                self.compiler.cpp(true);
                self.compiler.std("c++20");
            },
            StdVersion::CPP17 => {
                self.compiler.cpp(true);
                self.compiler.std("c++17");
            },
            StdVersion::CPP14 => {
                self.compiler.cpp(true);
                self.compiler.std("c++14");
            },
        }

        // compiles the thing into a single file
        self.compiler.compile_intermediates();
    }
}