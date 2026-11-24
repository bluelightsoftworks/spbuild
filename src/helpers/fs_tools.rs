use std::fs::{create_dir_all, exists, remove_dir_all};
use std::path::{Path, PathBuf};
use std::{fs, io};

fn _list_files(vec: &mut Vec<PathBuf>, path: &Path) -> io::Result<()> {
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let full_path = entry?.path();
            _list_files(vec, &full_path)?;
        }
    } else if path.is_file() {
        vec.push(path.to_path_buf());
    }
    Ok(())
}

/// Returns all file with a specified extension in a specific directory recursively
/// Searches for files in the files system
/// # Arguments
/// * `root` - The path to search from
/// * `extensions` - The extensions to search for
pub fn find_files(root: &PathBuf, extensions: &Vec<String>) -> Result<Vec<PathBuf>, String> {
    let files = list_files(root);

    #[allow(clippy::unnecessary_unwrap)]
    if files.is_err() {
        return Err(files.unwrap_err().to_string());
    }

    let filtered_files = filter_files(&files.unwrap(), root, extensions);

    Ok(filtered_files)
}

/// Lists files in the filesystem
pub fn list_files(root: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut abs_files = Vec::new();
    _list_files(&mut abs_files, root)?;

    // Convert to paths relative to `root` so callers can safely join(root, rel).
    let mut rel_files = Vec::with_capacity(abs_files.len());
    for p in abs_files {
        match p.strip_prefix(root) {
            Ok(rel) => rel_files.push(rel.to_path_buf()),
            Err(_) => rel_files.push(p),
        }
    }

    Ok(rel_files)
}

///
/// Returns all files of a specified extension found in the given list of files.
/// DOES NOT search for files in the filesystem; use `list_files` for that.
/// # Arguments
/// * `files` - A vector of PathBuf representing files to search through.
/// * `path` - The base path to join with found object files.
/// * `extension` - The extensions to keep
/// # Returns
/// * A vector of PathBuf containing the full paths to the found object files.
///
pub fn filter_files(files: &Vec<PathBuf>, path: &Path, extensions: &Vec<String>) -> Vec<PathBuf> {
    let mut found_files: Vec<PathBuf> = Vec::new();

    for file in files {
        let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");

        for available_ext in extensions {
            if ext.eq_ignore_ascii_case(available_ext) {
                let obj_path = path.join(file);
                found_files.push(obj_path);
            }
        }
    }

    found_files
}

/// deletes all specified directories *with their content* and creates new ones
pub fn override_dirs(dirs: &Vec<PathBuf>) -> Result<(), String> {
    for dir in dirs {
        _ = remove_dir_all(dir.as_path());

        if let Some(err) = create_dir_all(dir.as_path()).err() {
            return Err(err.to_string());
        }
    }

    Ok(())
}

/// checks if any of the dirs already exist
pub fn do_dirs_exists(dirs: Vec<PathBuf>) -> bool {
    for dir in dirs {
        if !exists(dir.as_path()).unwrap_or(false) {
            return false;
        }
    }
    true
}
