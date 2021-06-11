use flexi_logger::Logger;
use gitweb::options::Opt;
use gitweb::run;
use std::process::exit;
use structopt::StructOpt;

#[macro_use]
extern crate log;

fn main() {
    // Get the command line options.
    let opt = Opt::from_args();

    // Enable the logger with the output mode based on the given option.
    Logger::try_with_str(if opt.verbose { "debug" } else { "info" })
        .unwrap()
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));

    // Run the program using the given options.
    match run(opt) {
        Ok(_) => (),
        Err(err) => {
            info!("{}", err);
            exit(err.exit_code());
        }
    };
}
