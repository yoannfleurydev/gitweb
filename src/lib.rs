use regex::Regex;
use std::io::Result;
use std::process::exit;
use std::process::{Child, Command};

use crate::options::Opt;

#[macro_use]
extern crate log;

mod git;
pub mod options;

/// Enumeration to store exit codes. The first one, Success is by default set to 0
enum ExitCode {
    Success,
    NotInAGitRepository,
    NoRemoteMatching,
    NoRemoteAvailable,
    NotAbleToOpenSystemBrowser,
    BrowserNotAvailable,
}

const DEFAULT_REMOTE_ORIGIN: &str = "origin";
const BITBUCKET_HOSTNAME: &str = "bitbucket.org";

/// Function to open the browser using the system shell.
fn open_browser(browser: &str, url: &str) -> Result<Child> {
    Command::new(browser).arg(url).spawn()
}

pub fn get_remote_parts(url: &str) -> Result<(&str, &str)> {
    let re = Regex::new(r"((\w+://)|(git@))(.+@)*([\w\d.]+)(:[\d]+)?/*(:?)(.*)\.git/?").unwrap();
    let caps = re.captures(url).unwrap();

    let domain = caps.get(5).map_or("github.com", |m| m.as_str());
    let repository = caps.get(8).map_or("", |m| m.as_str());

    Ok((domain, repository))
}

pub fn run(opt: Opt) {
    // let logger = logger::Logger::new(opt.verbose);
    debug!("Verbose is active");

    let repo = git::get_repo().unwrap();

    // Get the tag to show in the browser. If the option is given, then the value
    // will be used as it is an alias for branch.
    let reference = if let Some(tag) = opt.tag {
        tag
    } else {
        // Get the branch to show in the browser. If the option is given, then, the
        // value will be used, else, the current branch is given, or master if
        // something went wrong.
        match opt.branch {
            Some(branch) => branch,
            None => {
                debug!("No branch given, getting current one");

                git::get_branch(&repo)
            }
        }
    };

    let remote_name = &opt
        .remote
        .unwrap_or_else(|| String::from(DEFAULT_REMOTE_ORIGIN));

    debug!("Getting remote for {}", remote_name);

    let optional_remote = match repo.find_remote(remote_name) {
        Ok(remote) => remote,
        Err(_) => {
            info!("No remote found for remote {}", remote_name);
            exit(ExitCode::NoRemoteMatching as i32);
        }
    };

    let remote_url = match optional_remote.url() {
        Some(remote) => remote,
        None => {
            info!("No remote URL available");
            exit(ExitCode::NoRemoteAvailable as i32);
        }
    };

    let (domain, repository) = get_remote_parts(remote_url).unwrap();

    let url = if let Some(commit) = opt.commit {
        format!(
            "https://{domain}/{repository}/{path}/{commit}",
            domain = domain,
            path = if domain == BITBUCKET_HOSTNAME {
                "commits"
            } else {
                "commit"
            },
            repository = repository,
            commit = commit
        )
    } else {
        format!(
            "https://{domain}/{repository}/{path}/{reference}",
            domain = domain,
            path = if domain == BITBUCKET_HOSTNAME {
                "src"
            } else {
                "tree"
            },
            repository = repository,
            reference = reference
        )
    };

    // If the option is available through the command line, open the given one
    match opt.browser {
        Some(option_browser) => {
            debug!("Browser {} given as option", option_browser);

            if option_browser == "" {
                println!("{}", url);
                exit(ExitCode::Success as i32);
            }

            if open_browser(&option_browser, &url).is_err() {
                info!("Unable to open the given browser: {}", option_browser);
                exit(ExitCode::BrowserNotAvailable as i32);
            };
        }
        None => {
            // Open the default web browser on the current system.
            match open::that(&url) {
                Ok(_) => debug!("Default browser is now open"),
                Err(_) => {
                    info!("Error while openning the default OS browser");
                    exit(ExitCode::NotAbleToOpenSystemBrowser as i32);
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
