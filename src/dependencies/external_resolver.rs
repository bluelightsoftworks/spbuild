use crate::helpers::version_tools::{is_valid_version_string, version_check};
use pkg_config::{Config, Library};
use std::path::{Path, PathBuf};

pub fn find_external_dependency(name: &String, version: &String) -> Result<Library, String> {
    let config = Config::new();

    if !is_valid_version_string(version) {
        Err(format!(
            "Invalid version string '{}'. Please use a valid semantic versioning format.",
            version
        ))?;
    }

    match config.probe(name.as_str()) {
        Ok(lib) => {
            if version_check(version, &lib.version) {
                Ok(lib)
            } else {
                Err(format!(
                    "Found version '{}' for dependencies '{}', but it does not satisfy the required version '{}'.",
                    lib.version, name, version
                ))
            }
        }
        Err(e) => Err(format!(
            "Failed to find external dependencies '{}': {}",
            name, e
        )),
    }
}

pub fn resolve_additional_include_directories(
    additional_includes: &Vec<PathBuf>,
    base_path: &Path,
) -> Result<Vec<PathBuf>, String> {
    let mut include_dirs: Vec<PathBuf> = Vec::new();

    // Check existence for each directory
    for inc in additional_includes {
        let abs_inc_path = base_path.join(inc);
        Console::log_verbose(
            format!("Adding additional include path: {}", abs_inc_path.display()).as_str(),
        );
        if !abs_inc_path.exists() {
            Console::log_fatal(
                format!(
                    "Additional include path does not exist: {}",
                    abs_inc_path.display()
                )
                .as_str(),
            );
            return Err("Failed to add additional include path".to_string());
        }
        include_dirs.push(abs_inc_path);
    }

    Ok(include_dirs)
}
