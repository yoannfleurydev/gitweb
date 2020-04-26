extern crate git2;
extern crate open;
extern crate regex;
extern crate structopt;

use regex::Regex;
use std::env;
use std::io::Result;
use std::process::exit;
use std::process::{Child, Command};
use structopt::StructOpt;

mod git;
mod logger;
mod options;

/// Function to open the browser using the system shell.
fn open_browser(browser: &String, url: &String) -> Result<Child> {
    Command::new(browser).arg(url).spawn()
}

/// Check if the given parameter is a port.
fn is_port(string: &str) -> bool {
    let re = Regex::new(r"^:\d{1,5}$").unwrap();

    re.is_match(string)
}

/// Function to remove the port if there is any.
fn remove_port(string: &str) -> String {
    let mut splits = string.split("/").collect::<Vec<&str>>();

    if splits.len() > 2 && is_port(splits[0]) {
        // Removing port
        splits.remove(0);
    }

    splits.join("/")
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
    BrowserNotAvailable,
}

fn main() {
    // Get the command line options
    let opt = options::Opt::from_args();
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
        }
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
    let repository = remove_port(caps.get(2).map_or("", |m| m.as_str()));

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

            if option_browser == "" {
                println!("{}", url);
                exit(ExitCode::Success as i32);
            }

            if open_browser(&option_browser, &url).is_err() {
                logger.print(
                    format!("Unable to open the given browser: {}", option_browser).as_str(),
                );
                exit(ExitCode::BrowserNotAvailable as i32);
            };
        }
        None => {
            match env::var(BROWSER) {
                // If the environment variable is available, open the web browser.
                Ok(browser) => {
                    if open_browser(&browser, &url).is_err() {
                        logger.print(
                            format!("Unable to open the given browser: {}", browser).as_str(),
                        );
                        exit(ExitCode::BrowserNotAvailable as i32);
                    }
                }
                // Else, open the default browser of the system.
                Err(e) => {
                    logger.verbose_print(
                        format!("{} variable not available : {}", BROWSER, e).as_str(),
                    );

                    // Open the default web browser on the current system.
                    match open::that(&url) {
                        Ok(_) => logger.verbose_print("Default browser is now open"),
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_port() {
        assert!(is_port(":80"));
    }

    #[test]
    fn test_is_not_port_it_is_a_path() {
        assert!(!is_port("/not_a_port"))
    }

    #[test]
    fn test_is_not_a_port_too_many_digits() {
        assert!(!is_port(":999999"))
    }

}
