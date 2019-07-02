extern crate git2;
extern crate open;
extern crate regex;
extern crate structopt;

use std::io::Result;
use regex::Regex;
use std::env;
use std::process::{Command,Child};
use std::process::exit;
use structopt::StructOpt;

mod git;
mod logger;

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
    branch: Option<String>,

    /// Set the browser
    ///
    /// If you set the browser option, it will override the other configuration.
    /// Here is the list by order of overrides: the --browser option given in
    /// the command line, then the environment variable $BROWSER on Linux or
    /// %BROWSER% on Windows (this is a non standard variable), then the default
    /// web browser on the syste
    #[structopt(short = "-B", long)]
    browser: Option<String>,

    /// Set the remote
    ///
    /// By default, the selected remote will be origin by convention. You can
    /// override this setting by  using this option.
    #[structopt(short, long)]
    remote: Option<String>,

    /// Set the verbosity of the command
    ///
    /// By settings this option, you will have more feedback on the output.
    #[structopt(short, long)]
    verbose: bool,
}

/// Function to open the browser using the system shell.
fn open_browser(browser: &String, url: &String) -> Result<Child> {
    Command::new(browser)
        .arg(url)
        .spawn()
}

const BROWSER: &str = "BROWSER";
const DEFAULT_REMOTE: &str = "origin";

/// Enumeration to store exit codes. The first one, Success is by default set to 0
enum ExitCode {
    Success,
    NotInAGitRepository,
    NoRemoteMatching,
    NoRemoteAvailable,
    NotAbleToOpenSystemBrowser,
    BrowserNotAvailable
}

fn main() {
    // Get the command line options
    let opt = Opt::from_args();
    let logger = logger::Logger::new(opt.verbose);
    logger.verbose_print("Verbose is active");

    // Check that the user is in a git repository.
    let repo = match git::get_repo() {
        Ok(repo) => repo,
        Err(_) => {
            logger.print("Command failed, please run command inside a git directory");
            exit(ExitCode::NotInAGitRepository as i32);
        }
    };

    // Get the branch to show in the browser. If the option is given, then, the
    // value will be used, else, the current branch is given, or master if
    // something went wrong.
    let branch = match opt.branch {
        Some(branch) => branch,
        None => {
            logger.verbose_print("No branch given, getting current one");

            git::get_branch(&repo, &logger)
        }
    };

    let remote_name = &opt.remote.unwrap_or(String::from(DEFAULT_REMOTE));

    logger.verbose_print(format!("Getting remote for {}", remote_name).as_str());

    let optional_remote = match repo.find_remote(remote_name) {
        Ok(remote) => remote,
        Err(_) => {
            logger.print(format!("No remote found for remote {}", remote_name).as_str());
            exit(ExitCode::NoRemoteMatching as i32);
        },
    };

    let remote_url = match optional_remote.url() {
        Some(remote) => remote,
        None => {
            logger.print("No remote URL available");
            exit(ExitCode::NoRemoteAvailable as i32);
        }
    };

    let re = Regex::new(r".*@(.*):(.*)\.git").unwrap();
    let caps = re.captures(remote_url).unwrap();

    let domain = caps.get(1).map_or("github.com", |m| m.as_str());
    let repository = caps.get(2).map_or("", |m| m.as_str());

    let url = format!(
        "https://{domain}/{repository}/tree/{branch}",
        domain = domain,
        repository = repository,
        branch = branch
    );

    // If the option is available through the command line, open the given one
    match opt.browser {
        Some(option_browser) => {
            logger.verbose_print(format!("Browser {} given as option", option_browser).as_str());

            if open_browser(&option_browser, &url).is_err() {
                logger.print(format!("Unable to open the given browser: {}", option_browser).as_str());
                exit(ExitCode::BrowserNotAvailable as i32);
            };
        },
        None => {
            match env::var(BROWSER) {
                // If the environment variable is available, open the web browser.
                Ok(browser) => {
                    if open_browser(&browser, &url).is_err() {
                        logger.print(format!("Unable to open the given browser: {}", browser).as_str());
                        exit(ExitCode::BrowserNotAvailable as i32);
                    }
                },
                // Else, open the default browser of the system.
                Err(e) => {
                    logger.verbose_print(format!("{} variable not available : {}", BROWSER, e).as_str());

                    // Open the default web browser on the current system.
                    match open::that(&url) {
                        Ok(_) => {
                            logger.verbose_print("Default browser is now open")
                        },
                        Err(_) => {
                            logger.print("Error while openning the default OS browser");
                            exit(ExitCode::NotAbleToOpenSystemBrowser as i32);
                        }
                    }
                }
            }
        }
    }

    exit(ExitCode::Success as i32);
}
