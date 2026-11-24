use std::fs::File;
use std::io::prelude::*;

use crate::InitArgs;
use crate::solution::{Dependency, Project, ProjectType};
use inquire::{Confirm, MultiSelect, Select, Text};

/// Creates a Project object interactively
fn create_project(available_projects: &Vec<Project>) -> Project {
    let name: String;
    let ptype;
    let platforms;
    let archs;
    let path;
    let path_buf;
    let mut deps: Vec<Dependency> = Vec::new();

    // Name
    match Text::new("Enter the project's name").prompt() {
        Ok(ans) => name = ans,
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    // Project Type
    let a_project_types = vec![
        ProjectType::Executable,
        ProjectType::DynamicLib,
        ProjectType::StaticLib,
    ];
    match Select::new("Chose the project's type", a_project_types).prompt() {
        Ok(choice) => {
            ptype = choice;
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    // Target platform
    let a_project_platforms = vec![Platform::Win, Platform::Linux, Platform::MacOS252];
    match MultiSelect::new("Choose the project's target platforms", a_project_platforms).prompt() {
        Ok(choice) => {
            platforms = choice;
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    // target architectures
    let a_project_archs = vec![
        Architecture::X86,
        Architecture::X64,
        Architecture::Arm,
        Architecture::Arm64,
        Architecture::Riscv64,
    ];
    match MultiSelect::new("Choose the project's target ISAs", a_project_archs).prompt() {
        Ok(choice) => {
            archs = choice;
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    // Project directory
    match Text::new(
        format!("Choose the project's directory (default: {})", {
            name.clone().to_ascii_lowercase()
        })
        .as_str(),
    )
    .prompt()
    {
        Ok(ans) => {
            // name of project is directory if none is found
            if ans.is_empty() {
                let mut pathified_name = name.clone();
                pathified_name.retain(|c| !c.is_ascii_alphanumeric() && c != '_');

                path = format!("./{}", pathified_name);
            } else {
                path = format!("./{}", ans);
            }

            path_buf = PathBuf::from(path);
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    // Dependency projects
    match Confirm::new("Do you wish to add any dependencies from the current solution?").prompt() {
        Ok(answer) => {
            if answer {
                match MultiSelect::new(
                    "Choose the dependencies projects",
                    available_projects.to_owned(),
                )
                .prompt()
                {
                    Ok(choices) => {
                        for el in &choices {
                            deps.push(el.clone().to_dependency());
                        }
                    }
                    Err(_) => {
                        Console::log_fatal("An unknown error occurred");
                        exit(1)
                    }
                }
            };
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    Project::new(
        name.as_str(),
        "0.0.1",
        ptype,
        archs,
        platforms,
        path_buf,
        deps,
        vec![],
        vec![],
    )
}

fn write_config(sol: Solution, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config = serde_yaml::to_string(&sol)?;

    let mut buffer = File::create(path)?;
    buffer.write_all(config.as_bytes())?;

    for project in sol.projects {
        let project_path = Path::new(&project.path);
        if !project_path.exists() {
            std::fs::create_dir_all(project_path)?;
        }
    }

    Ok(())
}

pub fn init_action(args: InitArgs) {
    let solution_name;
    let _path;

    if let Some(arg_path) = args.path.clone() {
        if !arg_path.exists() {
            Console::log_fatal(
                format!("The provided path does not exist: {}", arg_path.display()).as_str(),
            );
            exit(1);
        } else if arg_path.is_dir() {
            Console::log_warning("Provided path is a directory, using default filename 'sp.yaml'");
            _path = arg_path.join("sp.yaml");
        } else {
            _path = arg_path;
        }
    } else {
        _path = env::current_dir().unwrap().join("sp.yaml") //default path is exec path
    }

    // SOLUTION NAME
    match Text::new("Enter the solution's name").prompt() {
        Ok(name) => solution_name = name,
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    let mut projects: Vec<Project> = vec![];

    // Creates as many projects as user wants
    loop {
        let ans = Confirm::new("Do you want to add a new project?")
            .with_default(true)
            .prompt();

        match ans {
            Ok(true) => {
                projects.push(create_project(&projects));
                println!("Project created successfully!");
            }
            Ok(false) => break,
            Err(_) => {
                Console::log_fatal("An unknown error occurred");
                exit(1)
            }
        }
    }

    println!("Solution name: {}", solution_name);
    println!("Projects:");
    for project in &projects {
        println!("{}", project);
    }

    match Confirm::new("Does this look right")
        .with_default(true)
        .prompt()
    {
        Ok(true) => {
            println!("Nice! Creating sp repository")
        }
        Ok(false) => {
            println!(">> Aborting due to user input <<");
            exit(0);
        }
        Err(_) => {
            Console::log_fatal("An unknown error occurred");
            exit(1)
        }
    }

    let solution = Solution::new(solution_name.as_str(), projects);

    let res = write_config(solution, _path);
    if let Err(err) = res {
        Console::log_fatal(format!("An error occurred while writing config! {}", err).as_str());
        exit(1);
    }
}
