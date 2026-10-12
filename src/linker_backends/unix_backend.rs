use std::path::PathBuf;
use crate::linker_backends::common::LinkerBackend;

pub struct GnuLinkerBackend;


impl GnuLinkerBackend {
    pub fn new() -> Self {
        // Implement the necessary methods for the Gnu linker backend
        GnuLinkerBackend {}
    }
}

impl LinkerBackend for GnuLinkerBackend {
    fn link(&self, target: &str, files: Vec<PathBuf>, link_specific_options: Vec<String>) {
        todo!()
    }
}
