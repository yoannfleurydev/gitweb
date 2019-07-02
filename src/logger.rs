pub struct Logger {
    verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Logger {
        Logger { verbose: verbose }
    }

    pub fn verbose_print(&self, text: &str) {
        if self.verbose {
            println!("gitweb: {}", text);
        }
    }

    pub fn print(&self, text: &str) {
        println!("{}", text);
    }
}
