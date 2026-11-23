pub fn version_action() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
    const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
    const LICENSE: &str = env!("CARGO_PKG_LICENSE");

    Console::log_info(format!("{} version {}. Distributed under the {} license.", NAME, VERSION, LICENSE).as_str());
    Console::log_info(format!("{}", DESCRIPTION).as_str());
    Console::log_info("THIS SOFTWARE AND ANY RELATED DOCUMENTATION ARE PROVIDED \"AS IS\" WITHOUT WARRANTY OF ANY KIND.");
    Console::log_info(format!("More info available here: {}", HOMEPAGE).as_str());
}