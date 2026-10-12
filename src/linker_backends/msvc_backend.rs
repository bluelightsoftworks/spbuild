use std::path::PathBuf;
use crate::linker_backends::common::LinkerBackend;

pub struct MSVCLinkerBackend;


impl MSVCLinkerBackend {
    pub fn new() -> Self {
        // Implement the necessary methods for the MSVC linker backend
        MSVCLinkerBackend {}
    }
}

impl LinkerBackend for MSVCLinkerBackend {
    fn link(&self, target: &str, files: Vec<PathBuf>, link_specific_options: Vec<String>) {
        todo!()
    }
}
