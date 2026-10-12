//noinspection D
/// Compares two version strings for equality. This function compares two versions from the format "<mod>major.minor.patch".
/// Here are the available modifiers:
/// - "^": Compatible with version (e.g., "^1.2.3" is compatible with "1.3.0" but not "2.0.0").
/// - "~": Approximately equivalent to version (e.g., "~1.2.3" is approximately equivalent to "1.2.5" but not "1.3.0")
/// - ">" : Greater than version.
/// - "<" : Less than version.
/// - ">=": Greater than or equal to version.
/// - "<=": Less than or equal to version.
/// If no modifier is present, exact match is required.
///
/// # Arguments
/// * `required_version` - The version string that needs to be satisfied. contains the modifier.
/// * `provided_version` - The version string to check against the required version.
/// # Returns
/// * `true` if the versions are equal, `false` otherwise.
pub fn version_check(required_version: &str, provided_version: &str) -> bool {

    // Safely get the first character, if any
    let mut chars_iter = required_version.chars();
    let first_char_opt = chars_iter.next();
    if first_char_opt.is_none() {
        // Empty required_version cannot satisfy any constraint
        return false;
    }
    let first_char = first_char_opt.unwrap();
    let first_char_parse = first_char.to_string().parse::<i32>();
    let mut modifier_a: String = "==".to_string(); // Default to exact match

    // If changing the first character to an integer fails, it means it's not a digit
    if first_char_parse.is_err() {
        // Has modifier
        modifier_a = first_char.to_string();

        if modifier_a.eq(">") || modifier_a.eq("<") {
            // Check for two character modifiers (e.g., ">=" or "<=") safely
            if let Some(second_char) = chars_iter.next() {
                if second_char.eq(&'=') {
                    modifier_a.push(second_char);
                }
            }
        }
    }

    // Cleans version strings by removing modifiers
    let clean_required_version = required_version.replace(&['<', '>', '=', '^', '~'][..], "");
    let clean_provided_version = provided_version.replace(&['<', '>', '=', '^', '~'][..], "");


    // MY EEYYYYYYYYESSSSSS
    if modifier_a.eq("^") {                         // Compatible with version (same major)
        major_eq(clean_provided_version, clean_required_version)

    } else if modifier_a.eq("~") {                  // Approximately equivalent to version (same major and minor)
        major_eq(clean_provided_version.clone(), clean_required_version.clone())
            && minor_eq(clean_provided_version, clean_required_version)

    } else if modifier_a.eq(">") {                  // Greater than version
        // provided_version must be strictly greater than required_version
        greater_than(clean_provided_version, clean_required_version)

    } else if modifier_a.eq("<") {                  // Less than version
        // provided_version must be strictly less than required_version
        greater_than(clean_required_version, clean_provided_version)

    } else if modifier_a.eq(">=") {                 // Greater than or equal to version
        greater_than(clean_provided_version.clone(), clean_required_version.clone())
            || clean_provided_version.eq(&clean_required_version)

    } else if modifier_a.eq("<=") {                 // Less than or equal to version
        greater_than(clean_required_version.clone(), clean_provided_version.clone())
            || clean_provided_version.eq(&clean_required_version)

    } else {                                              // Exact match required (same major, minor, and patch)
        required_version.eq(provided_version)

    }

}

// Checks for major version equality
fn major_eq(version_a: String, version_b: String) -> bool {
    let major_a = version_a.split('.').next().unwrap_or("");
    let major_b = version_b.split('.').next().unwrap_or("");
    major_a == major_b
}

// Checks for minor version equality
fn minor_eq(version_a: String, version_b: String) -> bool {
    let parts_a: Vec<&str> = version_a.split('.').collect();
    let parts_b: Vec<&str> = version_b.split('.').collect();

    if parts_a.len() < 2 || parts_b.len() < 2 {
        return false;
    }

    parts_a[1] == parts_b[1]
}

// Checks if version_a is greater than version_b
fn greater_than(version_a: String, version_b: String) -> bool {
    let parts_a: Vec<u32> = version_a
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();
    let parts_b: Vec<u32> = version_b
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();

    // Not really elegant but it works
    let a0 = *parts_a.first().unwrap_or(&0);
    let a1 = *parts_a.get(1).unwrap_or(&0);
    let a2 = *parts_a.get(2).unwrap_or(&0);

    let b0 = *parts_b.first().unwrap_or(&0);
    let b1 = *parts_b.get(1).unwrap_or(&0);
    let b2 = *parts_b.get(2).unwrap_or(&0);

    (a0, a1, a2) > (b0, b1, b2)
}

#[cfg(test)]
mod tests {
    use super::version_check;

    #[test]
    fn exact_match_without_modifier() {
        assert!(version_check("1.2.3", "1.2.3"));
        assert!(!version_check("1.2.3", "1.2.4"));
    }

    #[test]
    fn caret_compatible_same_major() {
        assert!(version_check("^1.2.3", "1.0.0"));
        assert!(version_check("^1.2.3", "1.999.999"));
        assert!(!version_check("^1.2.3", "2.0.0"));
    }

    #[test]
    fn tilde_approximately_same_major_and_minor() {
        assert!(version_check("~1.2.3", "1.2.0"));
        assert!(version_check("~1.2.3", "1.2.999"));
        assert!(!version_check("~1.2.3", "1.3.0"));
        assert!(!version_check("~1.2.3", "2.2.3"));
    }

    #[test]
    fn greater_than() {
        assert!(!version_check(">1.2.3", "1.2.2"));
        assert!(!version_check(">1.2.3", "1.2.3"));
        assert!(version_check(">1.2.3", "1.2.4"));
    }

    #[test]
    fn less_than() {
        assert!(!version_check("<1.2.3", "1.2.3"));
        assert!(!version_check("<1.2.3", "1.2.4"));
        assert!(version_check("<1.2.3", "1.2.2"));
    }

    #[test]
    fn greater_than_or_equal() {
        assert!(version_check(">=1.2.3", "1.2.3"));
        assert!(!version_check(">=1.2.3", "1.2.2"));
        assert!(version_check(">=1.2.3", "1.2.4"));
    }

    #[test]
    fn less_than_or_equal() {
        assert!(version_check("<=1.2.3", "1.2.3"));
        assert!(version_check("<=1.2.3", "1.2.2"));
        assert!(!version_check("<=1.2.3", "1.2.4"));
    }

    #[test]
    fn missing_majors_minors() {
        // greater_than() fills out missing parts with 0s
        assert!(!version_check(">1", "0.9.9"));
        assert!(!version_check(">1", "1.0.0"));
        assert!(version_check(">1", "1.0.1"));
    }
}
