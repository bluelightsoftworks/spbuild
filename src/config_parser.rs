use std::path::Path;
use std::fs::read_to_string;

use crate::structs::solution::Solution;



pub fn parse_config(path: &Path) -> Result<Solution, String> {
    let contents = read_to_string(path)
        .map_err(|e| format!("Failed to read config file `{}': {}", path.display(), e))?;

    let solution = serde_saphyr::from_str(&contents)
        .map_err(|e| format!("Failed to parse config file `{}': {}", path.display(), e))?;

    Ok(solution)
}
