use std::path::PathBuf;
use crate::linker_backends::common::LinkerBackend;

pub struct LLVMElfLinkerBackend;


impl LLVMElfLinkerBackend {
    pub fn new() -> Self {
        // Implement the necessary methods for the Gnu linker backend
        LLVMElfLinkerBackend {}
    }
}

impl LinkerBackend for LLVMElfLinkerBackend {
    fn link(&self, target: &String, files: &Vec<PathBuf>, link_specific_options: &Vec<String>) {
        todo!()
    }
}
