mod solution;
mod config_parser;
mod target;

mod compiler_interfaces {
    pub mod common {
        include!("compiler_interfaces/common.rs");
    }
    pub mod msvc {
        include!("compiler_interfaces/msvc.rs");
    }
    pub mod gcc {
        include!("compiler_interfaces/gcc.rs");
    }
}

mod helpers {
    pub mod console {
        include!("helpers/console.rs");
    }

    pub mod version_tools{
        include!("helpers/version_tools.rs");
    }

    pub mod file_tools {
        include!("helpers/file_tools.rs");
    }
}

pub mod dependency_manager {
    pub mod local_resolve {
        include!("dependency_manager/dependency_resolver.rs");
    }
}

use std::env;
use std::path::PathBuf;
use clap::Parser;

// Basic helpers
use crate::helpers::console::Console;
use crate::config_parser::{parse_config};

// Compilation helpers
use crate::compiler_interfaces::common::Compiler;
use crate::compiler_interfaces::gcc::GccCompiler;
use crate::dependency_manager::local_resolve::{has_circular_dependency, resolve_project_build_inputs};

// Structs
use crate::solution::{ProjectType, Solution};
use crate::target::{Architecture, Platform};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_version_flag = true)]
struct Args {
    #[arg(short, long, help = "Path to the solution configuration file")]
    solution_path: Option<String>,

    #[arg(short, long, help = "The target platform for the build (e.g., linux, windows). Defaults to the current platform if not specified.")]
    platform: Option<String>,

    #[arg(short, long, help = "The target architecture for the build (e.g., x86, x64, arm, arm64). Defaults to the current architecture if not specified.")]
    architecture: Option<String>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Enable verbose output")]
    verbose: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Prints SPBuild version and exits")]
    version: bool,
}

fn config_file_check(config_path: &PathBuf) -> Result<PathBuf, String> {
    // Config file checks
    if !config_path.exists() {
        Console::log_fatal(format!("Configuration file not found: {}", config_path.display()).as_str());

        // Case config not found
        return Err("Configuration file not found".to_string());
    }

    if !config_path.is_file() {
        Console::log_warning(format!("Specified path is not a file: {}", config_path.display()).as_str());
        Console::log_warning("using default configuration file: spbuild.json\n");

        // Case path is not a file..
        //TODO: Check if the file exists in that folder?
        Ok(config_path.join("spbuild.json"))
    }
    else {
        Console::log_info(format!("Using solution configuration file: {}", &config_path.display()).as_str());

        // Case path is a file
        Ok(config_path.to_path_buf())
    }
}


//noinspection D
fn linux_build(args: Args, config_path: PathBuf, solution: Solution, target_arch: Architecture, target_platform: Platform) -> Result<(), ()> {
    let working_dir = config_path
        .parent()
        .expect("Config path has no parent")
        .to_path_buf();

    Console::log_success(format!("Successfully parsed solution: {}", solution.name).as_str());

// ===== PRE BUILD =====

    // Track what we've already compiled to avoid rebuilding the same dependency multiple times.
    let mut compiled_projects: Vec<String> = Vec::new();


    // Validate that all projects support the target architecture and platform before starting the build.
    for project in &solution.projects {
        // Check if the project supports the target architecture, platform..
        if !project.target_archs.contains(&target_arch) {
            Console::log_fatal(format!("Project {}: does not support target architecture {:?}", project.name, target_arch).as_str());
            return Err(());
        }
        if !project.target_platforms.contains(&target_platform) {
            Console::log_fatal(format!("Project {}: does not support target platform {:?}", project.name, target_platform).as_str());
            return Err(());
        }
    }

// ===== BUILD =====

    // Compiles each project (but checks which ones are compiled tho)
    for project in &solution.projects {

    // ===== COMPILATION  =====

        // Creates compiler for particular target
        let compiler = GccCompiler::new(target_arch.to_string(), target_platform.to_gcc_target_platform());

        // Resolve dependencies and include dirs.
        let inputs = match resolve_project_build_inputs(project, &solution, &working_dir, args.verbose) {
            Ok(v) => v,
            Err(e) => {
                Console::log_fatal(format!("Error resolving dependencies: {}", e).as_str());
                return Err(());
            }
        };

        // Build local deps first.
        for dep in &inputs.local_deps_in_order {
            if compiled_projects.iter().any(|n| n == &dep.name) {
                continue;
            }

            let res = compiler.compile_project(
                dep,
                &working_dir,
                Vec::new(),
                args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(format!("Error compiling dependency {}: {}", dep.name, e).as_str());
                return Err(());
            }

        // ===== LINKING  =====
            //TODO: [URGENT] Implement the additional_static_libs setting in the config for linking

            let res = compiler.link_project(
                dep,
                &solution,
                &working_dir,
                Vec::new(),
                args.verbose,
            );

            if let Err(e) = res {
                Console::log_fatal(format!("Error linking dependency {}: {}\n", dep.name, e).as_str());
                return Err(());
            }

            compiled_projects.push(dep.name.clone());
        }

        // Compile current project with resolved include dirs.
        let res = compiler.compile_project(
            &project,
            &working_dir,
            inputs.include_dirs.clone(),
            args.verbose,
        );

        if let Err(e) = res {
            Console::log_fatal(format!("Error compiling project: {}", e).as_str());
            return Err(());
        } else {
            Console::log_success("=== Project compiled successfully ===");
        }

        // Link current project.
        let mut link_inputs = inputs.dep_output_dirs.clone();
        // Keep the project's include dirs around as well in case the linker needs them later (e.g. for -L/-l).
        // For now, gcc.rs interprets these as directories to scan for `.o` files.
        link_inputs.extend(inputs.include_dirs.clone());

        let res = compiler.link_project(
            &project,
            &solution,
            &working_dir,
            link_inputs,
            args.verbose,
        );

        if let Err(e) = res {
            Console::log_fatal(format!("Error linking project: {}\n", e).as_str());
            return Err(());
        } else {
            Console::log_success("=== Project linked successfully ===");
        }

        // DLL resolution for windows targets
        if target_platform == Platform::Win && project.project_type == ProjectType::Executable {

            let gcc_target_arch = target_arch.to_gcc_target_arch();
            let gcc_target_platform = target_platform.to_gcc_target_platform();

            let abs_project_output_path = &working_dir
                .join("output")
                .join(format!("{}-{}", gcc_target_platform, gcc_target_arch)) // Target-specific output folder
                .join(&project.path);

            abs_project_output_path
                .canonicalize()
                .map_err(|_| {"Project Output Path not found"}).unwrap();
            let abs_exe_path = abs_project_output_path.join(&project.name).with_extension("exe");

            GccCompiler::resolve_dlls(&abs_exe_path, &args.verbose)
        }

        compiled_projects.push(project.name.clone());
    }
    Console::log_info("\n= BUILD COMPLETE =\n");
    Ok(())
}


fn print_version_and_exit() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

    Console::log_info(format!("===> {} version: {}\n", NAME, VERSION).as_str());
    Console::log_info(format!("{}\n", DESCRIPTION).as_str());
    Console::log_info(format!("More info at: {}", HOMEPAGE).as_str());
    std::process::exit(0);
}

fn main() {
    let args = Args::parse();

    if args.version {
        print_version_and_exit();
    }

    let config_path_string = args.solution_path.clone().unwrap_or_else(|| env::current_dir().unwrap().join("spbuild.json").display().to_string()) ;

    let mut config_path = PathBuf::from(&config_path_string);

    Console::log_info("===== SPBuild Starting =====");

    config_path = match config_file_check(&config_path) {
        Ok(path) => path,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let config = parse_config(&config_path).map_err(|e| {
        Console::log_fatal(format!("Failed to parse config: {}", e).as_str());
        Console::log_fatal("==== Aborting build ====");
    }).unwrap();


    Console::log_info("Detecting platform and architecture... ");
    let current_platform_str = env::consts::OS;
    let current_arch_str = env::consts::ARCH;
    Console::log_info(format!("Current platform/architecture: {}-{}", &current_platform_str, &current_arch_str).as_str());

    Console::log_info("\n= STARTING BUILD =\n");

    // TODO: Detect using `gcc -dumpmachine` if linux, and `cl.exe` if windows for more accurate target platform/arch.
    // String versions... For printing
    let target_platform_string = args.platform.clone().unwrap_or_else(|| current_platform_str.to_string());
    let target_architecture_string = args.architecture.clone().unwrap_or_else(|| current_arch_str.to_string());

    // Enums versions... for actually useful things
    let target_platform: Platform = match Platform::new(&target_platform_string) {
        Ok(p) => p,
        Err(e) => {
            Console::log_fatal(
                format!(
                    "Invalid target platform '{}': {}",
                    &target_platform_string, e
                )
                .as_str(),
            );
            Console::log_fatal("==== Aborting build ====");
            std::process::exit(1);
        }
    };

    let target_architecture: Architecture = match Architecture::new(&target_architecture_string) {
        Ok(a) => a,
        Err(e) => {
            Console::log_fatal(
                format!(
                    "Invalid target architecture '{}': {}",
                    &target_architecture_string, e
                )
                .as_str(),
            );
            Console::log_fatal("==== Aborting build ====");
            std::process::exit(1);
        }
    };


    Console::log_info(format!("Building for {}-{}", &target_platform_string, &target_architecture_string).as_str());

    for project in &config.projects {
        if has_circular_dependency(&project, &config, &mut Vec::new()) {
            Console::log_fatal(format!("Circular dependency detected in project: {}", project.name).as_str());
            Console::log_fatal("==== Aborting build ====");
            return;
        }
    }

    if current_platform_str == "windows" {
        Console::log_fatal("Windows platform detected. MSVC support is not yet implemented.");
        Console::log_fatal("==== Aborting build ====");
        //TODO : Call msvc functions
    }

    else if current_platform_str == "linux" {

        if linux_build(args, config_path, config, target_architecture, target_platform).is_err(){
            Console::log_fatal("==== Build failed ====");
            return;
        }
    }
}
