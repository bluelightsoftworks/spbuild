mod solution;
mod config_parser;
mod target;

mod compiler_interfaces {
    pub mod common {
        include!("compilers/common.rs");
    }
    pub mod gcc {
        include!("compilers/gcc.rs");
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

pub mod builders {
    pub mod gcc {
        include!("builders/builder.rs");
    }
}

pub mod dependency_manager {
    pub mod local_resolve {
        include!("dependency_manager/dependency_resolver.rs");
    }
}

pub mod commands {
    pub mod build_action {
        include!("actions/build_action.rs");
    }
}

use std::env;
use std::path::PathBuf;
use std::process::exit;
use clap::{Parser, Subcommand};

// Basic helpers
use crate::helpers::console::Console;
use crate::config_parser::{parse_config};

// Commands
use crate::commands::build_action::build_action;

// Compilation helpers
use crate::dependency_manager::local_resolve::{has_circular_dependency};

// Structs
use crate::solution::{Solution};
use crate::target::{Architecture, Platform};


// CLI Actions
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build
    {
        path: PathBuf,

        #[arg(short, long, help = "Enable verbose output")]
        verbose: bool,

        #[arg(short, long, help = "Specify a target architecture. Defaults to the current one")]
        architecture: Option<String>,

        #[arg(short, long, help = "Specify a target platform. Defaults to the current one")]
        platform: Option<String>,

    },
    Init,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {

    #[command(subcommand)]
    command: Commands,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Prints SPBuild version and exits")]
    version: bool,
}



//noinspection D
fn linux_build(args: Args, config_path: PathBuf, solution: Solution, target_arch: Architecture, target_platform: Platform) -> Result<(), ()> {
    let working_dir = config_path
        .parent()
        .expect("Config path has no parent")
        .to_path_buf();

    Console::log_success(format!("Successfully parsed solution: {}", solution.name).as_str());

    let mut gcc_builder = builders::gcc::GccBuilder::new(args, config_path, solution, target_arch, target_platform);

    gcc_builder.build_solution().map_err(|e| {
        Console::log_fatal(format!("Build failed: {}", e).as_str());
        Console::log_fatal("==== Aborting build ====");
    })?;

    Ok(())

}


fn version_action() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

    Console::log_info(format!("{} version {}\n", NAME, VERSION).as_str());
    Console::log_info(format!("{}\n", DESCRIPTION).as_str());
    Console::log_info(format!("More info at: {}", HOMEPAGE).as_str());
}

fn main() {
    let args = Args::parse();

    if args.version {
        version_action();
        exit(0);
    }

    match &args.command {
        Commands::Build { path, verbose, architecture, platform } => {
            build_action(args, &path, verbose, &architecture, &platform);
        }
        Commands::Init => {
            // Handle the init command
            Console::log_info("Init command received");
        }
    }
}
