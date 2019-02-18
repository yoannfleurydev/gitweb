pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Logger {
        Logger { verbose: verbose }
    }

    pub fn print(&self, text: &str) {
        if self.verbose {
            println!("gitweb: {}", text)
        }
    }
}
