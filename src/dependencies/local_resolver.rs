use crate::Console;
use crate::helpers::version_tools::version_check;
use crate::solution::{Project, Solution};

///
/// Finds and prints local dependencies of a given project within the solution.
/// # Arguments
/// * `project` - The project whose dependencies are to be resolved.
/// * `solution` - The solution containing all projects.
/// # Returns
/// * A Vec<project> object representing the resolved local dependencies.
///
pub fn find_local_deps(project: &Project, solution: &Solution) -> Vec<Project> {
    let mut found_deps: Vec<Project> = Vec::new();

    for dep in &project.dependencies {
        let required_version = &dep.version;

        // Checks each project for a matching name and version
        solution.projects.iter().for_each(|p| {
            if project.name == p.name {
                // Skip self
                return;
            }

            // Also checks for version equality
            Console::log_verbose(
                format!(
                    "Checking project {} version {} against dependencies {} version {}",
                    p.name, p.version, dep.name, dep.version
                )
                .as_str(),
            );
            Console::log_verbose(
                format!("Result of name check: {}", p.name.eq(&dep.name)).as_str(),
            );
            Console::log_verbose(
                format!(
                    "Result of version check: {}",
                    version_check(required_version, &p.version)
                )
                .as_str(),
            );

            // Version_eq checks if version B satisfies the criteria of version A. Here it checks if p.version satisfies dep.version
            if p.name.eq(&dep.name) && version_check(&dep.version, &p.version) {
                Console::log_info(
                    format!(
                        "Found local dependencies: {} version {} at path {}",
                        p.name,
                        p.version,
                        p.path.display()
                    )
                    .as_str(),
                );

                found_deps.push(p.clone());
            }
        });
    }

    found_deps
}

///
/// Checks for circular dependencies in the project graph.
/// Uses Recursion to traverse dependencies.
/// # Arguments
/// * `project` - The project to check for circular dependencies.
/// * `solution` - The solution containing all projects.
/// * `visited` - A mutable vector to keep track of visited projects during the check.
/// # Returns
/// * A boolean indicating whether the project specifies a circular dependency.
///
pub fn has_circular_dependency(
    project: &Project,
    solution: &Solution,
    visited: &mut Vec<String>,
) -> bool {
    if visited.contains(&project.name) {
        return true; // Circular dependencies detected
    }

    visited.push(project.name.clone());

    for dep in &project.dependencies {
        if let Some(dep_project) = solution.projects.iter().find(|p| p.name == dep.name)
            && has_circular_dependency(dep_project, solution, visited)
        {
            return true;
        }
    }

    visited.pop();
    false
}

/// Resolves local (in-solution) dependencies in a dependencies-first order.
///
/// # Returns
///
/// Notes:
/// - This is transitive: if A depends on B and B depends on C, the result for A is [C, B].
/// - Missing local deps are ignored here (they may be external/system deps).
pub fn resolve_local_deps(project: &Project, solution: &Solution) -> Vec<Project> {
    fn visit(
        current: &Project,
        solution: &Solution,
        out: &mut Vec<Project>,
        visiting: &mut Vec<String>,
    ) {
        if visiting.contains(&current.name) {
            // Circular dependencies should be checked earlier via has_circular_dependency.
            return;
        }
        visiting.push(current.name.clone());

        for dep in &current.dependencies {
            if let Some(dep_project) = solution.projects.iter().find(|p| p.name == dep.name) {
                // Best-effort version check; if it doesn't match, treat as non-local.
                if !version_check(&dep.version, &dep_project.version) {
                    continue;
                }

                visit(dep_project, solution, out, visiting);

                if !out.iter().any(|p| p.name == dep_project.name) {
                    out.push(dep_project.clone());
                }
            }
        }

        visiting.pop();
    }

    let mut out = Vec::new();
    let mut visiting = Vec::new();
    visit(project, solution, &mut out, &mut visiting);
    out
}
