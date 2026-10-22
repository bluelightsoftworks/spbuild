use std::path::PathBuf;
use std::process::Command;


use crate::linker_backends::common::LinkerBackend;
use crate::helpers::file_tools;
use crate::helpers::console;
use crate::helpers::console::Console;

pub struct GnuLinkerBackend{
    linker_path: PathBuf,

}


impl GnuLinkerBackend {
    pub fn new() -> Self {
        // Implement the necessary methods for the Gnu linker backend
        if !file_tools::check_file_exists(&PathBuf::from("/usr/bin/g++")) {
            Console::log_error("Error: Gnu linker not found at /usr/bin/g++");
            std::process::exit(1);
        }

        GnuLinkerBackend {
            linker_path: PathBuf::from("/usr/bin/g++"), // Assuming g++ is the linker for Gnu systems
        }
    }
}

impl LinkerBackend for GnuLinkerBackend {
    fn link(&self, target: &str, files: Vec<PathBuf>, link_specific_options: Vec<String>) {
        let mut command = Command::new(&self.linker_path);

        for option in link_specific_options {
            command.arg(option);
        }

        for file in files {
            command.arg(file);
        }

        command.arg("-o").arg(target);

        let output = command.output().expect("Failed to execute linker");

        if !output.status.success() {
            Console::log_error("Linking failed:");
            Console::log_error(&String::from_utf8_lossy(&output.stderr));
            std::process::exit(1);
        } else {
            Console::log_info("Linking succeeded.");
        }
    }
}
