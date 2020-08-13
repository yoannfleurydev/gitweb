use flexi_logger::Logger;
use gitweb::options::Opt;
use gitweb::run;
use structopt::StructOpt;

fn main() {
    // Get the command line options
    let opt = Opt::from_args();

    Logger::with_str(if opt.verbose { "debug" } else { "info" })
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
    run(opt);
}
