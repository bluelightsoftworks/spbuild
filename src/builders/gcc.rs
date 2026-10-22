use std::cmp::PartialEq;
use std::path::PathBuf;

use crate::Args;
use crate::compiler_interfaces::common::Compiler;
use crate::solution::{Solution, Project};
use crate::target::{Architecture, Platform};
use crate::compiler_interfaces::gcc::GccCompiler;
use crate::dependency_manager::local_resolve::resolve_project_build_inputs;
use crate::helpers::console::Console;
use crate::solution::ProjectType;

pub(crate) struct GccBuilder {
    args: Args,
    config_path: PathBuf,
    pub solution: Solution,
    pub target_arch: Architecture,
    pub target_platform: Platform,

    pub working_dir: PathBuf,
    pub compiled_projects: Vec<String>,

    pub compiler: GccCompiler
}



impl GccBuilder {
    pub fn new(args: Args, config_path: PathBuf, solution: Solution, target_arch: Architecture, target_platform: Platform) -> Self {
        let working_dir = config_path
            .parent()
            .expect("Config path has no parent")
            .to_path_buf();
        let compiled_projects = Vec::new();

        GccBuilder {
            args,
            config_path,
            solution,
            target_arch: target_arch.clone(),
            target_platform: target_platform.clone(),
            working_dir,
            compiled_projects,
            compiler: GccCompiler::new(target_arch.to_gcc_target_arch(), target_platform.to_gcc_target_platform()),
        }
    }

    pub fn pre_build_checks(&self) -> Result<(), String> {
        // Validate that all projects support the target architecture and platform before starting the build.
        for project in &self.solution.projects {
            // Check if the project supports the target architecture, platform..
            if !project.target_archs.contains(&self.target_arch) {
                Console::log_fatal(format!("Project {}: does not support target architecture {:?}", project.name, self.target_arch).as_str());
                return Err("Architecture not supported".to_string());
            }
            if !project.target_platforms.contains(&self.target_platform) {
                Console::log_fatal(format!("Project {}: does not support target platform {:?}", project.name, self.target_platform).as_str());
                return Err("Platform not supported".to_string());
            }
        }
        Ok(())
    }

    fn compile_project(&mut self, project: &Project) -> Result<(), String> {

        // Target string for path resolution
        let gcc_target_arch = self.target_arch.to_gcc_target_arch();
        let gcc_target_platform = self.target_platform.to_gcc_target_platform();
        let target_string = format!("{}-{}", gcc_target_platform, gcc_target_arch);

        // Resolve dependencies and include dirs.
        let inputs =
            match resolve_project_build_inputs(
                &project,
                &self.solution,
                &self.working_dir,
                &target_string,
                self.args.verbose) {
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
                &self.working_dir,
                Vec::new(),
                self.args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(format!("Error compiling dependency {}: {}", dep.name, e).as_str());
                return Err("Error compiling dependency: ".to_string());
            }

            let res = &self.compiler.link_project(
                dep,
                &self.solution,
                &self.working_dir,
                Vec::new(),
                self.args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(format!("Error linking dependency {}: {}\n", dep.name, e).as_str());
                return Err("Error linking dependency: ".to_string());
            }

            self.compiled_projects.push(dep.name.clone());
        }

        // Compile current project with resolved include dirs.
        let res = &self.compiler.compile_project(
            project,
            &self.working_dir,
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
        // For now, gcc.rs interprets these as directories to scan for `.o` files.
        link_inputs.extend(inputs.include_dirs.clone());

        let res = &self.compiler.link_project(
            &project,
            &self.solution,
            &self.working_dir,
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
        if &self.target_platform == Platform::Win && project.project_type == ProjectType::Executable {

            let abs_project_output_path = &self.working_dir
                .join("output")
                .join(&target_string) // Target-specific output folder
                .join(&project.path);

            abs_project_output_path
                .canonicalize()
                .map_err(|_| {"Project Output Path not found"})?;
            let abs_exe_path = abs_project_output_path.join(&project.name).with_extension("exe");

            if let Err(e) = GccCompiler::resolve_dlls(&abs_exe_path, &self.args.verbose) {
                Console::log_fatal(format!("Error resolving DLLs: {}", e).as_str());
                return Err(("Error resolving DLLs: ".to_string() + &e).to_string());
            }
        }

        self.compiled_projects.push(project.name.clone());
        Ok(())
    }

    pub fn build_solution(&mut self) -> Result<(), String> {
        self.pre_build_checks()?;

        // Compiles each project (but checks which ones are compiled tho)
        for project in self.solution.projects.clone() {

            if self.compiled_projects.iter().any(|n| n == &project.name) {
                Console::log_info(format!("Project {} already compiled. Skipping.", project.name).as_str());
                continue;
            }

            self.compile_project(&project)?;
        }

        Console::log_info("\n= BUILD COMPLETE =\n");
        Ok(())
    }
}