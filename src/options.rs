extern crate structopt;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
// Rename all will use the name of the field
#[structopt(rename_all = "kebab-case")]
pub struct Opt {
    /// Set the branch
    ///
    /// By setting the branch, you can override the default behavior that will
    /// set the branch to the current one in the repository. If something went
    /// wrong with the current one, it will set the value to master.
    #[structopt(short, long)]
    pub branch: Option<String>,

    /// Set a commit
    ///
    /// By setting a commit, you can override the default behavior that will
    /// set the branch to the current one in the repository.
    #[structopt(short, long)]
    pub commit: Option<String>,

    /// Set the browser
    ///
    /// If you set the browser option, it will override the other configuration.
    /// Here is the list by order of overrides: the --browser option given in
    /// the command line, then the environment variable $BROWSER on Linux or
    /// %BROWSER% on Windows (this is a non standard variable), then the default
    /// web browser on the system
    /// If you give an empty string to browser option, the program will only
    /// print the remote URL into the stdout.
    #[structopt(short = "-B", long)]
    pub browser: Option<String>,

    /// Set the remote
    ///
    /// By default, the selected remote will be origin by convention. You can
    /// override this setting by  using this option.
    #[structopt(short, long)]
    pub remote: Option<String>,

    /// Set the verbosity of the command
    ///
    /// By settings this option, you will have more feedback on the output.
    #[structopt(short, long)]
    pub verbose: bool,
}
