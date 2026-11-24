use crate::BuildArgs;
use crate::compiler_interfaces::common::Compiler;
use crate::compiler_interfaces::gcc::GccCompiler;
use crate::dependency_manager::local_resolver::resolve_additional_includes;
use crate::helpers::console::Console;
use crate::helpers::file_tools::{do_dirs_exists, override_dirs};
use crate::solution::ProjectType;
use crate::solution::{Project, Solution};
use crate::target::{Architecture, Platform};
use inquire::Confirm;
use std::path::PathBuf;
use std::process::exit;

pub struct GccBuilder {
    args: BuildArgs,
    pub solution: Solution,
    pub target_arch: Architecture,
    pub target_platform: Platform,

    // needs to be a file
    pub config_path: PathBuf,
    pub compiled_projects: Vec<String>,

    pub compiler: GccCompiler,
}

impl GccBuilder {
    pub fn new(
        args: BuildArgs,
        config_path: PathBuf,
        solution: Solution,
        target_arch: Architecture,
        target_platform: Platform,
    ) -> Self {
        let compiled_projects = Vec::new();

        let mut cpath = config_path;

        if !cpath.is_file() {
            Console::log_error("Passed config path is not a file. Guessing it must be sp.yaml");
            cpath.push("sp.yaml");
        }

        GccBuilder {
            args,
            solution,
            target_arch: target_arch.clone(),
            target_platform: target_platform.clone(),
            config_path: cpath,
            compiled_projects,
            compiler: GccCompiler::new(
                target_arch.to_gcc_target_arch(),
                target_platform.to_gcc_target_platform(),
            ),
        }
    }

    /// Validate that all passed projects support the target architecture and platform
    fn _validate_target_support(&self) -> Result<(), String> {
        for project in &self.solution.projects {
            // arch check
            if !project.target_archs.contains(&self.target_arch) {
                Console::log_fatal(
                    format!(
                        "Project {}: does not support target architecture {:?}",
                        project.name, self.target_arch
                    )
                    .as_str(),
                );
                return Err("Architecture not supported".to_string());
            }

            // plat check
            if !project.target_platforms.contains(&self.target_platform) {
                Console::log_fatal(
                    format!(
                        "Project {}: does not support target platform {:?}",
                        project.name, self.target_platform
                    )
                    .as_str(),
                );
                return Err("Platform not supported".to_string());
            }
        }
        Ok(())
    }

    /// Creates the directories required for building the solution
    fn _create_build_dirs(&self) -> Result<(), String> {
        // build dir
        let build = self
            .config_path
            .parent()
            .unwrap()
            .join("build")
            .join(format!(
                "{}-{}",
                self.target_platform.to_gcc_target_platform(),
                self.target_arch.to_gcc_target_arch()
            ));
        let artifacts = build.join("artifacts");
        let output = build.join("output");

        let dirs = vec![build, artifacts, output.to_path_buf()];

        if do_dirs_exists(vec![output]) && !self.args.override_output {
            match Confirm::new(
                "Output directory already exist. Do you wish to continue and overwrite them?",
            )
            .with_default(false)
            .prompt()
            {
                Ok(true) => {}
                Ok(false) => {
                    Console::log_info(">> Aborting build due to user input <<");
                    exit(0)
                }
                Err(_) => {
                    Console::log_fatal(">> Aborting build due to user output failure <<");
                }
            }
        }

        if let Some(err) = override_dirs(&dirs).err() {
            Console::log_fatal(format!("Failed to create build directories: {}", err).as_str());
            exit(1);
        };

        Ok(())
    }

    /// Runs pre-build checks and preparation
    fn pre_build(&self) -> Result<(), String> {
        // validate target arch/plat
        if let Some(err) = self._validate_target_support().err() {
            return Err(err);
        }

        // create build dirs
        if let Some(err) = self._create_build_dirs().err() {
            return Err(err);
        }

        Ok(())
    }

    fn compile_project(&mut self, project: &Project) -> Result<(), String> {
        // Target string for path resolution
        let gcc_target_arch = self.target_arch.to_gcc_target_arch();
        let gcc_target_platform = self.target_platform.to_gcc_target_platform();
        let target_string = format!("{}-{}", gcc_target_platform, gcc_target_arch);

        // Resolve dependencies and include dirs.
        let inputs = match resolve_additional_includes(
            project,
            &self.solution,
            &self.config_path,
            &target_string,
            self.args.verbose,
        ) {
            Ok(v) => v,
            Err(e) => {
                Console::log_fatal(format!("Error resolving dependencies: {}", e).as_str());
                return Err("Error resolving dependencies: ".to_string());
            }
        };

        // Build local deps first.
        for dep in &inputs.local_deps_in_order {
            if self.compiled_projects.iter().any(|n| n == &dep.name) {
                continue;
            }

            let res = &self.compiler.compile_project(
                dep,
                &self.config_path,
                Vec::new(),
                self.args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(
                    format!("Error compiling dependency {}: {}", dep.name, e).as_str(),
                );
                return Err("Error compiling dependency: ".to_string());
            }

            let res = &self.compiler.link_project(
                dep,
                &self.solution,
                &self.config_path,
                Vec::new(),
                self.args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(
                    format!("Error linking dependency {}: {}\n", dep.name, e).as_str(),
                );
                return Err("Error linking dependency: ".to_string());
            }

            self.compiled_projects.push(dep.name.clone());
        }

        // Compile current project with resolved include dirs.
        let res = &self.compiler.compile_project(
            project,
            &self.config_path,
            inputs.include_dirs.clone(),
            self.args.verbose,
        );

        if let Err(e) = res {
            Console::log_fatal(format!("Error compiling project: {}", e).as_str());
            return Err("Error compiling project: ".to_string());
        } else {
            Console::log_success("=== Project compiled successfully ===");
        }

        // Link current project.
        let mut link_inputs = inputs.dep_output_dirs.clone();
        // Keep the project's include dirs around as well in case the linker needs them later (e.g. for -L/-l).
        // For now, gcc_builder interprets these as directories to scan for `.o` files.
        link_inputs.extend(inputs.include_dirs.clone());

        let res = &self.compiler.link_project(
            project,
            &self.solution,
            &self.config_path,
            link_inputs,
            self.args.verbose,
        );

        if let Err(e) = res {
            Console::log_fatal(format!("Error linking project: {}\n", e).as_str());
            return Err("Error linking project: ".to_string());
        } else {
            Console::log_success("=== Project linked successfully ===");
        }

        // DLL resolution for windows targets
        if self.target_platform == Platform::Win && project.project_type == ProjectType::Executable
        {
            let abs_project_output_path = &self
                .config_path
                .join("build")
                .join(&target_string) // Target-specific output folder
                .join(&project.path);

            abs_project_output_path
                .canonicalize()
                .map_err(|_| "Project Output Path not found")?;
            let abs_exe_path = abs_project_output_path
                .join(&project.name)
                .with_extension("exe");

            if let Err(e) = GccCompiler::resolve_dlls(&abs_exe_path, &self.args.verbose) {
                Console::log_fatal(format!("Error resolving DLLs: {}", e).as_str());
                return Err(("Error resolving DLLs: ".to_string() + &e).to_string());
            }
        }

        self.compiled_projects.push(project.name.clone());
        Ok(())
    }

    pub fn build_solution(&mut self) -> Result<(), String> {
        if let Some(pre_build_err) = self.pre_build().err() {
            return Err(pre_build_err.to_string());
        }

        // Compiles each project (but checks which ones are compiled tho)
        for project in self.solution.projects.clone() {
            if self.compiled_projects.iter().any(|n| n == &project.name) {
                Console::log_info(
                    format!("Project {} already compiled. Skipping.", project.name).as_str(),
                );
                continue;
            }

            self.compile_project(&project)?;
        }

        Console::log_info("\n= BUILD COMPLETE =\n");
        Ok(())
    }
}
