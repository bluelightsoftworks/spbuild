use std::path::PathBuf;

pub trait LinkerBackend {

    fn link(&self, target: &String, files: &Vec<PathBuf>, link_specific_options: &Vec<String>);
}