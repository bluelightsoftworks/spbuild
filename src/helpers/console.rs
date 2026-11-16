use colored::Colorize;

pub struct Console;

impl Console {
    // Basic standardized logging functions
    pub fn log_verbose(message: &str, verbose: bool) {
        if verbose {
            println!("{}{}",
                    "v- ".italic().dimmed(),
                     message.italic().dimmed());
        }
    }

    pub fn log_info(message:  &str) {
        println!("{}", message);
    }

    pub fn log_warning(message: &str) {
        println!("{} > {}",
                 "[WARNING]".yellow().bold(),
                 message);
    }
    pub fn log_success(message: &str) {
        println!("{} > {}",
                 "[SUCCESS]".green().bold(),
                 message);
    }

    pub fn log_error(message: &str) {
        eprintln!("{} > {}",
                  "[ERROR]".red(),
                  message);
    }

    pub fn log_fatal(message: &str) {
        eprintln!("{} > {}",
                  "[FATAL]".bold().red(),
                  message);
        eprint!("\n");
    }
}