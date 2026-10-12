mod config_parser;
mod builder;
mod linker;
mod linker_backends {
    pub mod common {
        include!("linker_backends/common.rs");
    }

    pub mod unix_backend;
    pub mod msvc_backend;

}

pub mod structs {
    pub mod target {
        include!("structs/target.rs");
    }

    pub mod solution {
        include!("structs/solution.rs");
    }
}

pub mod enums {
    use serde::Deserialize;

    include!("enums.rs");
}

pub mod helpers {
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
use std::ffi::OsStr;
use std::path::{PathBuf, Path};
use clap::Parser;
use crate::builder::Builder;
// Basic helpers
use crate::helpers::console::Console;

// Compilation helpers
use crate::dependency_manager::local_resolve::{has_circular_dependency, resolve_project_build_inputs};
use crate::helpers::file_tools::list_files;
// Structs
use crate::structs::solution::{Project, Solution};
use crate::structs::target::{Architecture, Platform, Target};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_version_flag = true)]
struct Args {
    #[arg(short, long, help = "Path to the solution configuration file")]
    solution_path: Option<String>,

    #[arg(short, long, help = "The target platform for the build (e.g., linux, win). Defaults to the current platform if not specified.")]
    platform: Option<String>,

    #[arg(short, long, help = "The target architecture for the build (e.g., x86, x86_64, arm, aarch64). Defaults to the current architecture if not specified.")]
    architecture: Option<String>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Enable verbose output")]
    verbose: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Prints SPBuild version and exits")]
    version: bool,
}

/// <summary>
/// Checks the existence of the solution's configuration path
/// <summary/>
fn config_file_check(config_path: &Path) -> Result<PathBuf, String> {
    // Config file checks
    if !config_path.exists() {
        Console::log_error(format!("Configuration file not found: {}", config_path.display()).as_str());

        // Case config not found
        return Err("Configuration file not found".to_string());
    }

    if !config_path.is_file() {
        Console::log_warning(format!("Specified path is not a file: {}", config_path.display()).as_str());
        Console::log_warning("using default configuration file: sp.yaml\n");

        // Case path is not a file: treat as directory and look for default config
        let default_config = config_path.join("sp.yaml");

        if !default_config.exists() {
            Console::log_error(
                format!(
                    "Default configuration file not found in directory: {}",
                    default_config.display()
                )
                .as_str(),
            );
            return Err("Configuration file not found".to_string());
        }

        if !default_config.is_file() {
            Console::log_error(
                format!(
                    "Default configuration path is not a file: {}",
                    default_config.display()
                )
                .as_str(),
            );
            return Err("Configuration file is not a regular file".to_string());
        }

        Console::log_info(
            format!(
                "Using default solution configuration file: {}",
                default_config.display()
            )
            .as_str(),
        );

        Ok(default_config)
    } else {
        Console::log_info(format!("Using solution configuration file: {}", &config_path.display()).as_str());

        // Case path is a file
        Ok(config_path.to_path_buf())
    }
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

fn print_version_header() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");

    Console::log_info(format!("{} version: {}\n", NAME, VERSION).as_str());
}

fn main() {
    let args = Args::parse();

    if args.version {
        print_version_and_exit();
    }

    let config_path_string = args.solution_path.clone().unwrap_or_else(|| env::current_dir().unwrap().join("sp.yaml").display().to_string()) ;

    let mut config_path = PathBuf::from(&config_path_string);
    let mut solution: Solution;

    Console::log_info("===== SPBuild Starting =====");

    config_path = match config_file_check(&config_path) {
        Ok(path) => path,
        Err(_) => {
            std::process::exit(1);
        }
    };

    let solution_root = config_path
        .parent()
        .unwrap_or_else(|| Path::new(".").as_ref())
        .to_path_buf();

    if config_path.extension().and_then(OsStr::to_str) == Some("yaml") || config_path.extension().and_then(OsStr::to_str) == Some("yml") {
        solution = config_parser::parse_config(&config_path).map_err(|e| {
            Console::log_error(format!("Failed to parse config: {}", e).as_str());
            Console::log_error("===== Aborting build =====");
            std::process::exit(1);
        }).unwrap();
    } else {

    // Unsupported config file format
        Console::log_error(format!("Unknown configuration file format: {}", config_path.display()).as_str());
        Console::log_error("===== Aborting build =====");
        std::process::exit(1);
    }


    Console::log_info("Detecting platform and architecture... ");
    let current_platform_str = env::consts::OS;
    let current_arch_str = env::consts::ARCH;
    Console::log_info(format!("Current platform/architecture: {}-{}", &current_platform_str, &current_arch_str).as_str());

    // Normalize the current platform string so it matches what `Platform::new` expects.
    // For example, `env::consts::OS` returns "windows", while the parser may expect "win".
    let normalized_platform_str = match current_platform_str {
        "windows" => "win",
        other => other,
    };

    Console::log_info("\n===== STARTING BUILD =====\n");

    // TODO: Detect using `gcc -dumpmachine` if linux, and `cl.exe` if windows for more accurate target platform/arch.
    // String versions... For printing
    let target_platform_string = args
        .platform
        .clone()
        .unwrap_or_else(|| normalized_platform_str.to_string());
    let target_architecture_string = args
        .architecture
        .clone()
        .unwrap_or_else(|| current_arch_str.to_string());

    // Enums versions... for actually useful things
    let target_platform: Platform = match Platform::new(&target_platform_string) {
        Ok(p) => p,
        Err(e) => {
            Console::log_error(
                format!(
                    "Invalid target platform '{}': {}",
                    &target_platform_string, e
                )
                .as_str(),
            );
            Console::log_error("==== Aborting build ====");
            std::process::exit(1);
        }
    };

    let target_architecture: Architecture = match Architecture::new(&target_architecture_string) {
        Ok(a) => a,
        Err(e) => {
            Console::log_error(
                format!(
                    "Invalid target architecture '{}': {}",
                    &target_architecture_string, e
                )
                .as_str(),
            );
            Console::log_error("==== Aborting build ====");
            std::process::exit(1);
        }
    };

    let target_triple = match target_platform {
        Platform::Linux => format!(
            "{}-unknown-{}",
            target_architecture,
            target_platform.to_gcc_target_platform()
        ),
        Platform::Win => format!("{}-pc-windows-msvc", target_architecture),
        Platform::MacOS252 => format!("{}-apple-darwin", target_architecture),
        Platform::Unknown => format!("{}-unknown-unknown", target_architecture),
    };

    let host_triple = match Target::current_host_triple() {
        Ok(triple) => triple,
        Err(e) => {
            Console::log_error(format!("Failed to determine host triple: {}", e).as_str());
            Console::log_error("==== Aborting build ====");
            std::process::exit(1);
        }
    };


    Console::log_info(format!("Building for {}", &target_triple).as_str());
    for project in &solution.projects {

        if has_circular_dependency(project, &solution, &mut Vec::new()) {
            Console::log_error(format!("Circular dependency detected in project: {}", project.name).as_str());
            Console::log_error("==== Aborting build ====");
            return;
        }
    }

    solution.create_compile_list();

    build_solution(
        &solution,
        &target_triple,
        target_architecture,
        target_platform,
        &host_triple,
        solution_root,
        args.verbose,
    );
}


fn build_solution(
    solution: &Solution,
    target_triple: &String,
    target_architecture: Architecture,
    target_platform: Platform,
    host_triple: &String,
    solution_root: PathBuf,
    verbose: bool,
) {

    for project in &solution.compile_queue {

        let mut builder = Builder::new(
            target_triple,
            host_triple,
            &project.project_type
        );

        let result = compile_project(
            project,
            solution,
            &solution_root,
            target_triple,
            verbose,
            &mut builder,
        );

        #[allow(clippy::unnecessary_unwrap)]
        if result.is_err() {
            Console::log_error(format!("Failed to compile project {}: {}", project.name, result.unwrap_err()).as_str());
            Console::log_error("===== Aborting build =====");
            std::process::exit(1);
        }

    }
}

fn compile_project(
    project: &Project,
    solution: &Solution,
    solution_root: &PathBuf,
    target_triple: &String,
    verbose: bool,
    builder: &mut Builder,
) -> Result<(), String>
{
    let cpp_version = project.cpp_version.clone().unwrap_or_default();

    let build_inputs = resolve_project_build_inputs(
        project,
        solution,
        solution_root,
        target_triple,
        verbose,
    )
    .map_err(|e| e.to_string())?;

    let abs_output_path = solution_root.join("output").join(target_triple).join(&project.path);
    let abs_project_path = solution_root.join(&project.path);

    // Create the list of files to compile in a project
    let mut file_list = list_files(&abs_project_path).map_err(|e| {
        Console::log_error(format!("Failed to list files in project {}: {}", project.name, e).as_str());
        Console::log_error("===== Aborting build =====");
        std::process::exit(1);
    }).unwrap();

    // Only compile cpp, cc, cxx and c files
    file_list.retain(|file| {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        ext.eq_ignore_ascii_case("c") || ext.eq_ignore_ascii_case("cpp") ||
            ext.eq_ignore_ascii_case("cc") || ext.eq_ignore_ascii_case("cxx")
    });

    for file in file_list {
        builder.add_file(&file);
    }


    // Adds include paths from the project itself and its local dependencies.
    for include in &build_inputs.include_dirs {
        builder.add_include(include);
    }


    builder.set_cpp_version(&cpp_version);

    builder.build(&abs_output_path)
    
}