mod config_parser;
mod solution;
mod target;

mod compiler_interfaces {
    pub mod common {
        include!("compilers/common.rs");
    }
    pub mod gcc {
        include!("compilers/gcc_compiler.rs");
    }
}

mod helpers {
    pub mod console {
        include!("helpers/console.rs");
    }

    pub mod version_tools {
        include!("helpers/version_tools.rs");
    }

    pub mod file_tools {
        include!("helpers/fs_tools.rs");
    }
}

pub mod builders {
    pub mod gcc {
        include!("builders/gcc_builder.rs");
    }
}

pub mod dependencies {
    include!("dependencies/dependency_resolver.rs");

    pub mod local_resolver {
        include!("dependencies/local_resolver.rs");
    }

    pub mod external_resolver {
        use crate::helpers::console::Console;

        include!("dependencies/external_resolver.rs");
    }

    pub mod online_resolver {
        include!("dependencies/online_resolver.rs");
    }
}

pub mod actions {
    include!("actions/build_action.rs");
    include!("actions/init_action.rs");
    include!("actions/version_action.rs");
}

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::exit;

// Basic helpers
use crate::helpers::console::Console;

// Commands
use crate::actions::build_action;
use crate::actions::init_action;
use crate::actions::version_action;

#[derive(clap::Args, Debug, Clone)]
pub struct BuildArgs {
    path: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Override output directory if it already exists")]
    override_output: bool,

    #[arg(long, global = true, action = clap::ArgAction::SetTrue, help = "Prints SPBuild version and exits")]
    verbose: bool,

    #[arg(
        short,
        long,
        help = "Specify a target architecture. Defaults to the current one"
    )]
    architecture: Option<String>,

    #[arg(
        short,
        long,
        help = "Specify a target platform. Defaults to the current one"
    )]
    platform: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct InitArgs {
    path: Option<PathBuf>,
}

// CLI Actions
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    Build(BuildArgs),
    Init(InitArgs),
    Version,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None, disable_version_flag = true)]
pub struct Args {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Args::parse();

    match args.clone().command {
        Commands::Build(build_args) => {
            Console::set_verbose(build_args.verbose);
            build_action(build_args)
        },

        Commands::Init(init_args) => init_action(init_args),
        Commands::Version => {
            version_action();
            exit(0);
        }
    }
}
