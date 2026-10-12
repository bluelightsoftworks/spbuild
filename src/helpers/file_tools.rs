use std::path::{Path, PathBuf};
use std::{fs, io};

fn collect_files_recursive(files: &mut Vec<PathBuf>, path: &Path) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let full_path = entry.path();
            collect_files_recursive(files, &full_path)?;
        }
    } else if path.is_file() {
        files.push(path.to_path_buf());
    }
    Ok(())
}

pub fn list_files(root: &Path) -> io::Result<Vec<PathBuf>> {
    // Canonicalize the root path to resolve any relative prefixes (like "./")
    // and symlinks, ensuring a clean absolute base path.
    let canonical_root = root.canonicalize()?;

    let mut absolute_files = Vec::new();
    collect_files_recursive(&mut absolute_files, &canonical_root)?;

    Ok(absolute_files)
}

///
/// Returns all the object files (.o) found in the given list of files.
/// DOES NOT search for files in the filesystem; use `list_files` for that.
/// # Arguments
/// * `files` - A vector of PathBuf representing files to search through.
/// * `path` - The base path to join with found object files.
/// # Returns
/// * A vector of PathBuf containing the full paths to the found object files.
///
pub fn find_object_files(files: &Vec<PathBuf>, path: &Path) -> Vec<PathBuf> {
    let mut found_object_files: Vec<PathBuf> = Vec::new();

    for file in files {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext.eq_ignore_ascii_case("o") {
            let obj_path = path.join(&file);
            found_object_files.push(obj_path);
        }
    }

    found_object_files
}

pub fn create_directory_if_not_exists(path: &Path) -> io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}