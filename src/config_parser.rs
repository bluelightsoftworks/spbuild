use std::path::Path;
use std::fs::read_to_string;

use crate::solution::Solution;

pub fn parse_config(path: &Path) -> Result<Solution, String> {

    let unserialized_string = read_to_string(path);

    // Error handling
    if (&unserialized_string).is_err() {
        return Err(format!("Failed to read config file: {}", path.display()));
    }
    
    // Result<String> -> String -> &str -> Config
    let contents = read_to_string(path)
        .map_err(|e| format!("Failed to read config file `{}': {}", path.display(), e))?;

    let solution: Solution = serde_yaml::from_str(&contents)
        .map_err(|e| format!("Failed to parse config file `{}': {}", path.display(), e))?;

    Ok(solution)
}
