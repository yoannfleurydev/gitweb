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
fn open_browser(browser: &str, url: &str) -> Result<Child> {
    Command::new(browser).arg(url).spawn()
}

fn get_remote_parts(url: &str) -> Result<(&str, &str)> {
    let re =
        Regex::new(r"((\w+://)|(git@))(.+@)*([\w\d\.]+)(:[\d]+){0,1}/*(:?)(.*)\.git/?").unwrap();
    let caps = re.captures(url).unwrap();

    let domain = caps.get(5).map_or("github.com", |m| m.as_str());
    let repository = caps.get(8).map_or("", |m| m.as_str());

    Ok((domain, repository))
}

const BROWSER: &str = "BROWSER";
const DEFAULT_REMOTE_ORIGIN: &str = "origin";
const BITBUCKET_HOSTNAME: &str = "bitbucket.org";

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

    let remote_name = &opt
        .remote
        .unwrap_or_else(|| String::from(DEFAULT_REMOTE_ORIGIN));

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

    let parts = get_remote_parts(remote_url).unwrap();

    let url = if let Some(commit) = opt.commit {
        format!(
            "https://{domain}/{repository}/{path}/{commit}",
            domain = parts.0,
            path = if parts.0 == BITBUCKET_HOSTNAME {"commits"} else {"commit"},
            repository = parts.1,
            commit = commit
        )
    } else {
        format!(
            "https://{domain}/{repository}/{path}/{branch}",
            domain = parts.0,
            path = if parts.0 == BITBUCKET_HOSTNAME {"src"} else {"tree"},
            repository = parts.1,
            branch = branch
        )
    };

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
    fn test_without_ssh_git_url_parts() {
        let parts = get_remote_parts("git@github.com:yoannfleurydev/gitweb.git").unwrap();

        assert_eq!(parts.0, "github.com");
        assert_eq!(parts.1, "yoannfleurydev/gitweb");
    }

    #[test]
    fn test_with_ssh_and_multiple_subgroups_git_url_parts() {
        let parts =
            get_remote_parts("ssh://git@gitlab.com/group/subgroup/subsubgroup/design-system.git")
                .unwrap();

        assert_eq!(parts.0, "gitlab.com");
        assert_eq!(parts.1, "group/subgroup/subsubgroup/design-system");
    }

    #[test]
    fn test_with_ssh_and_port_git_url_parts() {
        let parts = get_remote_parts("ssh://user@host.xz:22/path/to/repo.git/").unwrap();

        assert_eq!(parts.0, "host.xz");
        assert_eq!(parts.1, "path/to/repo");
    }

    #[test]
    fn test_with_http_and_port_git_url_parts() {
        let parts = get_remote_parts("http://host.xz:80/path/to/repo.git/").unwrap();

        assert_eq!(parts.0, "host.xz");
        assert_eq!(parts.1, "path/to/repo");
    }

    #[test]
    fn test_with_http_git_url_parts() {
        let parts = get_remote_parts("https://host.xz/path/to/repo.git/").unwrap();

        assert_eq!(parts.0, "host.xz");
        assert_eq!(parts.1, "path/to/repo");
    }
}
