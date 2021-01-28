use regex::Regex;
use thiserror::Error;

use crate::options::Opt;

#[macro_use]
extern crate log;

mod git;
pub mod options;

const BITBUCKET_HOSTNAME: &str = "bitbucket.org";
const GITHUB_HOSTNAME: &str = "github.com";
const GITLAB_HOSTNAME: &str = "gitlab.com";
const GITEA_HOSTNAME: &str = "gitea.io";

#[derive(Debug, Eq, Error, PartialEq, Clone)]
pub enum Issue {
    #[error("Command failed, please run command inside a git directory")]
    NotInAGitRepository,
    #[error("No matching remote url found for '{0}' remote name")]
    NoRemoteMatching(String),
    #[error("No remote available")]
    NoRemoteAvailable,
    #[error("Not able to open system browser")]
    NotAbleToOpenSystemBrowser,
    #[error("Unable to open browser '{0}'")]
    BrowserNotAvailable(String),
    #[error("Unable to get remote parts, please open an issue as it might come from the code")]
    UnableToGetRemoteParts,
    #[error("Unknown provider")]
    UnknownProvider,
}

pub struct Success;
type Result = core::result::Result<Success, Issue>;

impl Issue {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::NotInAGitRepository => 1,
            Self::NoRemoteMatching(..) => 2,
            Self::NoRemoteAvailable => 3,
            Self::NotAbleToOpenSystemBrowser => 4,
            Self::BrowserNotAvailable(..) => 5,
            Self::UnableToGetRemoteParts => 6,
            Self::UnknownProvider => 7,
        }
    }
}

enum GitProvider {
    GitHub,
    GitLab,
    Bitbucket,
    Gitea,
}

impl Default for GitProvider {
    fn default() -> Self {
        Self::GitHub
    }
}

impl GitProvider {
    fn hostname(&self) -> String {
        match self {
            Self::GitHub => GITHUB_HOSTNAME,
            Self::GitLab => GITLAB_HOSTNAME,
            Self::Bitbucket => BITBUCKET_HOSTNAME,
            Self::Gitea => GITEA_HOSTNAME,
        }
        .to_string()
    }
}

pub struct RemoteParts {
    domain: String,
    repository: String,
}

struct MergeRequestParts {
    path: String,
    tail: String,
}

const DEFAULT_REMOTE_ORIGIN: &str = "origin";

fn get_remote_parts(url: &str) -> anyhow::Result<RemoteParts> {
    let re: Regex = Regex::new(r"((\w+://)|(git@))(.+@)*(?P<domain>[\w\d.]+)(:[\d]+)?/*(:?)(?P<repository>[^.]*)(\.git)?(/)?$").unwrap();

    let caps = re
        .captures(url)
        .ok_or_else(|| ())
        .map_err(|_| Issue::UnableToGetRemoteParts)?;

    let domain = caps
        .name("domain")
        .map_or(GitProvider::GitHub.hostname(), |m| m.as_str().to_string());
    let repository = caps
        .name("repository")
        .map_or("".to_string(), |m| m.as_str().to_string());

    Ok(RemoteParts { domain, repository })
}

fn get_merge_request_parts(domain: &str) -> anyhow::Result<MergeRequestParts, Issue> {
    match domain {
        GITHUB_HOSTNAME => Ok(MergeRequestParts {
            path: "pulls".to_string(),
            tail: "".to_string(),
        }),
        GITLAB_HOSTNAME => Ok(MergeRequestParts {
            path: "-/merge_requests".to_string(),
            tail: "".to_string(),
        }),
        BITBUCKET_HOSTNAME => Ok(MergeRequestParts {
            path: "pull-requests".to_string(),
            tail: "".to_string(),
        }),
        GITEA_HOSTNAME => Ok(MergeRequestParts {
            path: "pulls".to_string(),
            tail: "".to_string(),
        }),
        _ => Err(Issue::UnknownProvider)
    }
}

pub fn run(opt: Opt) -> Result {
    // let logger = logger::Logger::new(opt.verbose);
    debug!("Verbose mode is active");

    let repo = git::get_repo()?;

    // Get the tag to show in the browser. If the option is given, then the value
    // will be used as it is an alias for branch.
    let reference = if let Some(tag) = opt.tag {
        tag
    } else {
        // Get the branch to show in the browser. If the option is given, then, the
        // value will be used, else, the current branch is given, or master if
        // something went wrong.
        opt.branch.unwrap_or_else(|| {
            debug!("No branch given, getting current one");
            git::get_branch(&repo)
        })
    };

    let remote_name = &opt
        .remote
        .unwrap_or_else(|| String::from(DEFAULT_REMOTE_ORIGIN));

    debug!("Getting remote url for '{}' remote name", remote_name);

    let optional_remote = repo
        .find_remote(remote_name)
        .map_err(|_| Issue::NoRemoteMatching(remote_name.clone()))?;

    let remote_url = optional_remote
        .url()
        .ok_or_else(|| ())
        .map_err(|_| Issue::NoRemoteAvailable)?;

    let RemoteParts { domain, repository } = get_remote_parts(remote_url).unwrap();

    let (path, tail) = if let Some(commit) = opt.commit {
        let path = if domain == GitProvider::Bitbucket.hostname() {
            "commits"
        } else {
            "commit"
        };

        (path, commit)
    } else {
        let path = if domain == GitProvider::Bitbucket.hostname() {
            "src"
        } else {
            "tree"
        };

        (path, reference)
    };

    let (path, tail) = if opt.merge_request {
        debug!("Getting merge request parts for domain '{}'", domain);
        let MergeRequestParts { path, tail } = get_merge_request_parts(&domain).unwrap();
        (path.as_str().to_owned(), tail.as_str().to_owned())
    } else {
        (path.to_string(), tail.to_string())
    };

    // Generate the requested url that has to be opened in the browser
    let url = generate_url(&domain, &repository, &path, &tail);

    // If the option is available through the command line, open the given one
    match opt.browser {
        Some(option_browser) => {
            debug!("Browser '{}' given as option", option_browser);

            if option_browser == "".to_string() {
                println!("{}", url);
            }

            open::with(&url, &option_browser)
                .map_err(|_| Issue::BrowserNotAvailable(option_browser))?;

            Ok(Success)
        }
        None => {
            // Open the default web browser on the current system.
            match open::that(&url) {
                Ok(_) => {
                    debug!("Default browser is now opened");
                    Ok(Success)
                }
                Err(_) => Err(Issue::NotAbleToOpenSystemBrowser),
            }
        }
    }
}

fn generate_url(domain: &str, repository: &String, path: &String, tail: &String) -> String {
    format!(
        "https://{domain}/{repository}/{path}/{tail}",
        domain = domain,
        path = path,
        repository = repository,
        tail = tail
    )
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_without_ssh_git_and_without_extension_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("git@github.com:yoannfleurydev/gitweb").unwrap();

        assert_eq!(domain, "github.com");
        assert_eq!(repository, "yoannfleurydev/gitweb");
    }

    #[test]
    fn test_without_ssh_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("git@github.com:yoannfleurydev/gitweb.git").unwrap();

        assert_eq!(domain, "github.com");
        assert_eq!(repository, "yoannfleurydev/gitweb");
    }

    #[test]
    fn test_with_ssh_and_multiple_subgroups_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("ssh://git@gitlab.com/group/subgroup/subsubgroup/design-system.git")
                .unwrap();

        assert_eq!(domain, "gitlab.com");
        assert_eq!(repository, "group/subgroup/subsubgroup/design-system");
    }

    #[test]
    fn test_with_ssh_and_port_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("ssh://user@host.xz:22/path/to/repo.git/").unwrap();

        assert_eq!(domain, "host.xz");
        assert_eq!(repository, "path/to/repo");
    }

    #[test]
    fn test_with_http_and_port_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("http://host.xz:80/path/to/repo.git/").unwrap();

        assert_eq!(domain, "host.xz");
        assert_eq!(repository, "path/to/repo");
    }

    #[test]
    fn test_with_http_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("https://host.xz/path/to/repo.git/").unwrap();

        assert_eq!(domain, "host.xz");
        assert_eq!(repository, "path/to/repo");
    }

    #[test]
    fn test_get_merge_request_parts_with_github() {
        let MergeRequestParts { path, tail } =
            get_merge_request_parts(GITHUB_HOSTNAME).unwrap();

        assert_eq!(path, "pulls");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_gitlab() {
        let MergeRequestParts { path, tail } =
            get_merge_request_parts(GITLAB_HOSTNAME).unwrap();

        assert_eq!(path, "-/merge_requests");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_bitbucket() {
        let MergeRequestParts { path, tail } =
            get_merge_request_parts(BITBUCKET_HOSTNAME).unwrap();

        assert_eq!(path, "pull-requests");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_gitea() {
        let MergeRequestParts { path, tail } =
            get_merge_request_parts(GITEA_HOSTNAME).unwrap();

        assert_eq!(path, "pulls");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_unknown_provider() {
        let result = get_merge_request_parts("host.xz");

        assert_eq!(result.err(), Some(Issue::UnknownProvider));
    }
}
