use colored::Colorize;
use std::sync::atomic::AtomicBool;

pub struct Console;

static VERBOSE: AtomicBool = AtomicBool::new(false);

impl Console {
    
pub fn set_verbose(verbose: bool) {
        VERBOSE.store(verbose, std::sync::atomic::Ordering::Relaxed);
    }
    
    // Basic standardized logging functions
    pub fn log_verbose(message: &str) {
        if VERBOSE.load(std::sync::atomic::Ordering::Relaxed) {
            println!("{}{}", "v- ".italic().dimmed(), message.italic().dimmed());
        }
    }

    pub fn log_info(message: &str) {
        println!("{}", message);
    }

    pub fn log_warning(message: &str) {
        println!("{} > {}", "[WARNING]".yellow().bold(), message);
    }
    pub fn log_success(message: &str) {
        println!("{} > {}", "[SUCCESS]".green().bold(), message);
    }

    pub fn log_error(message: &str) {
        eprintln!("{} > {}", "[ERROR]".red(), message);
    }

    pub fn log_fatal(message: &str) {
        eprintln!("{} > {}", "[FATAL] \n".bold().red(), message);
    }
}
