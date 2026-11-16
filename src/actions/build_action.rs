use std::env::consts::{ARCH, OS};
use std::path::{Path, PathBuf};
use std::process::exit;
use crate::config_parser::parse_config;
use crate::dependency_manager::local_resolve::has_circular_dependency;
use crate::helpers::console::Console;
use crate::Args;
use crate::target::{Architecture, Platform};
use crate::solution::Solution;

fn config_file_check(config_path: &PathBuf) -> Result<PathBuf, String> {
    // Config file checks
    if !config_path.exists() {
        // Case config not found
        return Err("Configuration file not found".to_string());
    }

    if !config_path.is_file() {
        Console::log_warning(format!("Specified path is not a file: {}", config_path.display()).as_str());
        Console::log_warning("using default configuration file: sp.yaml\n");

        // Case path is not a file: treat as directory and look for default config
        let default_config = config_path.join("sp.yaml");

        // Case even default path doesn't exist
        if !default_config.exists() {
            return Err(format!("Default configuration file not found in directory: {}", default_config.display()));
        }

        // Case default path is actually a directory (why would u do that tho)
        if !default_config.is_file() {
            return Err(format!("Default configuration path is not a file: {}", default_config.display()));
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

fn linux_build(args: Args, config_path: PathBuf, solution: Solution, target_arch: Architecture, target_platform: Platform) -> Result<(), ()> {
    // Implementation for Linux build
    Ok(())
}

pub fn build_action(args: Args, config_path: &PathBuf, verbose: &bool, arch_arg: &Option<String>, platform_arg: &Option<String>){

    // ---- config file
    let mut config_file_path: PathBuf = config_path.clone();

    config_file_path = match config_file_check(&config_path) {
        Ok(path) => path,
        Err(err) => {
            Console::log_fatal(format!("Failed to open file - {}", err).as_str());
            exit(1);
        }
    };


    // ---- actual config object
    let config = parse_config(&config_file_path)
        .map_err(|err| {
            Console::log_fatal(format!("Failed to parse file - {err}").as_str());
            exit(1);
        }).unwrap();


    // ---- host config
    let host_platform = OS;
    let host_arch = ARCH;
    Console::log_verbose(format!("Current platform/Architecture: {}-{}", host_platform, host_arch).as_str(), *verbose);

    let normal_host_platform = match host_platform { // Normalizes for `Platform::new()`
        "windows" => "win",
        other => other
    };


    // ---- target config
    let trgt_platform_str = platform_arg.clone().unwrap_or_else(|| normal_host_platform.to_string());
    let trgt_arch_str = arch_arg.clone().unwrap_or_else(|| host_arch.to_string());

    let target_platform: Platform = match Platform::new(trgt_platform_str.as_str()) {
        Ok(p) => p,
        Err(err) => {
            Console::log_fatal(format!("Invalid target platform \"{}\" - \"{}\"", trgt_platform_str, err).as_str());
            exit(1);
        }
    };

    let target_arch: Architecture = match Architecture::new(trgt_arch_str.as_str()) {
        Ok(a) => a,
        Err(err) => {
            Console::log_fatal(format!("Invalid target architecture \"{}\" - \"{}\"", trgt_arch_str, err).as_str());
            exit(1);
        }
    };

    // ---- check circular deps
    for project in &config.projects {
        if has_circular_dependency(&project, &config, &mut Vec::new()) {
            Console::log_fatal(format!("Circular dependency detected in project {}", project.name).as_str());
            exit(1);
        }
    }


    // ---- start building
    Console::log_info(format!("Building for {}-{}", trgt_platform_str, trgt_arch_str).as_str());

    if host_platform != "linux"{
        Console::log_fatal(format!("Current host platform '{}' is not supported yet. Only Linux is supported.", host_platform).as_str());
        exit(1);
    }

    linux_build(args, config_path.clone(), config, target_arch, target_platform);
    

}