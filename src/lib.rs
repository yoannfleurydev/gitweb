use git2::Config;
use git_url_parse::GitUrl;
use thiserror::Error;

use crate::options::Opt;

#[macro_use]
extern crate log;

mod git;
pub mod options;
mod template;

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
    #[error(
        "Unable to fill {{{placeholder}}} in {key}, the available placeholders are {available}"
    )]
    InvalidTemplate {
        key: String,
        placeholder: String,
        available: String,
    },
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
            Self::InvalidTemplate { .. } => 8,
        }
    }
}

#[derive(Default)]
enum GitProvider {
    #[default]
    GitHub,
    GitLab,
    Bitbucket,
    Gitea,
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

impl RemoteParts {
    // The last segment of the repository path.
    fn repo(&self) -> &str {
        self.repository.rsplit('/').next().unwrap_or_default()
    }

    // The segment just before the repo, so scm/PROJ/repo has PROJ as owner.
    fn owner(&self) -> Option<&str> {
        self.repository.rsplit('/').nth(1)
    }
}

struct MergeRequestParts {
    path: String,
    tail: String,
}

const DEFAULT_REMOTE_ORIGIN: &str = "origin";

fn get_remote_parts(url: &str) -> anyhow::Result<RemoteParts> {
    let giturl = GitUrl::parse(url).map_err(|_| Issue::UnableToGetRemoteParts)?;

    let domain = giturl
        .host()
        .map_or(GitProvider::GitHub.hostname(), |m| m.to_string());

    let path = giturl
        .path()
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("/");

    // don't want the .git part, but only as a suffix: user.github.io is a name
    let repository = path.strip_suffix(".git").unwrap_or(&path).to_string();

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
        _ => Err(Issue::UnknownProvider),
    }
}

pub fn run(opt: Opt) -> Result {
    // let logger = logger::Logger::new(opt.verbose);
    debug!("Verbose mode is active");

    let repo = git::get_repo()?;

    let target = Target::from_options(&opt, || {
        debug!("No branch given, getting current one");
        git::get_branch(&repo)
    });

    let remote_name = &opt
        .remote
        .unwrap_or_else(|| String::from(DEFAULT_REMOTE_ORIGIN));

    debug!("Getting remote url for '{}' remote name", remote_name);

    let optional_remote = repo
        .find_remote(remote_name)
        .map_err(|_| Issue::NoRemoteMatching(remote_name.clone()))?;

    let remote_url = optional_remote
        .url()
        .map_err(|_| Issue::NoRemoteAvailable)?;

    // A git config that cannot be read must not prevent from opening the
    // built-in urls, as it was the case before the url templates.
    let config = repo
        .config()
        .or_else(|e| {
            warn!(
                "Unable to read the git config, using the built-in urls: {}",
                e
            );
            Config::new()
        })
        .map_err(|_| Issue::NotInAGitRepository)?;

    // Generate the requested url that has to be opened in the browser
    let url = build_url(&target, remote_url, &config)?;
    debug!("Opening '{}'", url);

    // If the option is available through the command line, open the given one
    match opt.browser {
        Some(option_browser) => {
            debug!("Browser '{}' given as option", option_browser);

            if option_browser == *"" {
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

// What to show in the browser for the repository.
#[derive(Debug, PartialEq)]
enum Target {
    Branch(String),
    Tag(String),
    Commit(String),
    MergeRequest,
}

impl Target {
    // Get the target from the options: the merge requests override the commit,
    // that overrides the tag, that overrides the branch. Without any of them,
    // the current branch is shown.
    fn from_options(opt: &Opt, current_branch: impl FnOnce() -> String) -> Self {
        if opt.merge_request {
            Self::MergeRequest
        } else if let Some(commit) = &opt.commit {
            Self::Commit(commit.clone())
        } else if let Some(tag) = &opt.tag {
            Self::Tag(tag.clone())
        } else {
            Self::Branch(opt.branch.clone().unwrap_or_else(current_branch))
        }
    }

    // The key of the template of the target, as in gitweb.<host>.<key>.
    fn template_key(&self) -> &'static str {
        match self {
            Self::Branch(..) => "branch-template",
            Self::Tag(..) => "tag-template",
            Self::Commit(..) => "commit-template",
            Self::MergeRequest => "merge-request-template",
        }
    }

    // The placeholders that are only available in the template of the target.
    fn placeholders(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Branch(branch) => vec![
                ("branch", branch.clone()),
                ("branch_encoded", template::encode(branch)),
            ],
            Self::Tag(tag) => vec![("tag", tag.clone()), ("tag_encoded", template::encode(tag))],
            Self::Commit(commit) => vec![("commit", commit.clone())],
            Self::MergeRequest => vec![],
        }
    }
}

// Build the url of the target for the repository behind the remote url.
fn build_url(
    target: &Target,
    remote_url: &str,
    config: &Config,
) -> core::result::Result<String, Issue> {
    let parts = get_remote_parts(remote_url).map_err(|_| Issue::UnableToGetRemoteParts)?;

    let key = format!("gitweb.{}.{}", parts.domain, target.template_key());

    if let Some(template) = template::find(config, &parts.domain, target.template_key()) {
        debug!("Using the '{}' template from {}", template, key);

        let mut placeholders = vec![
            ("host", parts.domain.clone()),
            ("repository", parts.repository.clone()),
        ];
        // A repository without owner leaves the {owner} placeholder unavailable
        placeholders.extend(parts.owner().map(|owner| ("owner", owner.to_string())));
        placeholders.push(("repo", parts.repo().to_string()));
        placeholders.extend(target.placeholders());

        return template::render(&template, &placeholders).map_err(|placeholder| {
            Issue::InvalidTemplate {
                key,
                placeholder,
                available: placeholders
                    .iter()
                    .map(|(name, _)| format!("{{{name}}}"))
                    .collect::<Vec<String>>()
                    .join(", "),
            }
        });
    }

    debug!("No {} configured, using the built-in url", key);

    let RemoteParts { domain, repository } = parts;

    let (path, tail) = match target {
        Target::Branch(reference) | Target::Tag(reference) => {
            let path = if domain == GitProvider::Bitbucket.hostname() {
                "src"
            } else {
                "tree"
            };

            (path.to_string(), reference.clone())
        }
        Target::Commit(commit) => {
            let path = if domain == GitProvider::Bitbucket.hostname() {
                "commits"
            } else {
                "commit"
            };

            (path.to_string(), commit.clone())
        }
        Target::MergeRequest => {
            debug!("Getting merge request parts for domain '{}'", domain);
            let MergeRequestParts { path, tail } = get_merge_request_parts(&domain)?;
            (path, tail)
        }
    };

    Ok(generate_url(&domain, &repository, &path, &tail))
}

fn generate_url(domain: &str, repository: &str, path: &str, tail: &str) -> String {
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
    use clap::Parser;
    use git2::ConfigLevel;
    use tempfile::TempDir;

    fn branch(name: &str) -> Target {
        Target::Branch(name.to_string())
    }

    fn tag(name: &str) -> Target {
        Target::Tag(name.to_string())
    }

    fn commit(sha: &str) -> Target {
        Target::Commit(sha.to_string())
    }

    // Build the url without any gitweb configuration.
    fn build_builtin_url(target: &Target, remote_url: &str) -> core::result::Result<String, Issue> {
        build_url(target, remote_url, &Config::new().unwrap())
    }

    #[test]
    fn test_build_url_with_github_branch() {
        let url = build_builtin_url(&branch("main"), "git@github.com:yoannfleurydev/gitweb.git");

        assert_eq!(
            url,
            Ok("https://github.com/yoannfleurydev/gitweb/tree/main".to_string())
        );
    }

    #[test]
    fn test_build_url_with_github_commit() {
        let url = build_builtin_url(
            &commit("abc123"),
            "git@github.com:yoannfleurydev/gitweb.git",
        );

        assert_eq!(
            url,
            Ok("https://github.com/yoannfleurydev/gitweb/commit/abc123".to_string())
        );
    }

    #[test]
    fn test_build_url_with_bitbucket_branch() {
        let url = build_builtin_url(&branch("main"), "git@bitbucket.org:org/repo.git");

        assert_eq!(
            url,
            Ok("https://bitbucket.org/org/repo/src/main".to_string())
        );
    }

    #[test]
    fn test_build_url_with_bitbucket_commit() {
        let url = build_builtin_url(&commit("abc123"), "git@bitbucket.org:org/repo.git");

        assert_eq!(
            url,
            Ok("https://bitbucket.org/org/repo/commits/abc123".to_string())
        );
    }

    #[test]
    fn test_build_url_with_unknown_host_branch() {
        let url = build_builtin_url(
            &branch("main"),
            "ssh://git@bitbucket.company.com:7999/org/repo.git",
        );

        assert_eq!(
            url,
            Ok("https://bitbucket.company.com/org/repo/tree/main".to_string())
        );
    }

    #[test]
    fn test_build_url_with_merge_requests() {
        let cases = [
            (
                "git@github.com:org/repo.git",
                "https://github.com/org/repo/pulls/",
            ),
            (
                "git@gitlab.com:org/repo.git",
                "https://gitlab.com/org/repo/-/merge_requests/",
            ),
            (
                "git@bitbucket.org:org/repo.git",
                "https://bitbucket.org/org/repo/pull-requests/",
            ),
            (
                "git@gitea.io:org/repo.git",
                "https://gitea.io/org/repo/pulls/",
            ),
        ];

        for (remote_url, expected) in cases {
            let url = build_builtin_url(&Target::MergeRequest, remote_url);

            assert_eq!(url, Ok(expected.to_string()), "for {remote_url}");
        }
    }

    #[test]
    fn test_build_url_with_unknown_host_merge_request() {
        let url = build_builtin_url(&Target::MergeRequest, "git@host.xz:org/repo.git");

        assert_eq!(url, Err(Issue::UnknownProvider));
    }

    // Build a config from the content of git config files, one per level.
    fn leveled_config(files: &[(ConfigLevel, &str)]) -> (TempDir, Config) {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::new().unwrap();

        for (index, (level, content)) in files.iter().enumerate() {
            let path = dir.path().join(format!("config-{index}"));
            std::fs::write(&path, content).unwrap();
            config.add_file(&path, *level, false).unwrap();
        }

        (dir, config)
    }

    // Build a config from the content of a local git config file.
    fn local_config(content: &str) -> (TempDir, Config) {
        leveled_config(&[(ConfigLevel::Local, content)])
    }

    const BITBUCKET_SERVER_CONFIG: &str = r#"
[gitweb "bitbucket.company.com"]
    branch-template = https://{host}/projects/{owner}/repos/{repo}/browse?at=refs%2Fheads%2F{branch}
"#;

    #[test]
    fn test_build_url_with_branch_template() {
        let (_dir, config) = local_config(BITBUCKET_SERVER_CONFIG);

        let url = build_url(
            &branch("main"),
            "ssh://git@bitbucket.company.com:7999/PROJ/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok(
                "https://bitbucket.company.com/projects/PROJ/repos/repo/browse?at=refs%2Fheads%2Fmain"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_build_url_with_branch_template_and_https_remote() {
        let (_dir, config) = local_config(BITBUCKET_SERVER_CONFIG);

        let url = build_url(
            &branch("main"),
            "https://bitbucket.company.com/scm/PROJ/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok(
                "https://bitbucket.company.com/projects/PROJ/repos/repo/browse?at=refs%2Fheads%2Fmain"
                    .to_string()
            )
        );
    }

    #[test]
    fn test_build_url_with_template_host_in_another_case() {
        let (_dir, config) = local_config(
            r#"
[gitweb "Bitbucket.Company.com"]
    branch-template = https://{host}/{repo}/{branch}
"#,
        );

        let url = build_url(
            &branch("main"),
            "git@bitbucket.company.com:PROJ/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok("https://bitbucket.company.com/repo/main".to_string())
        );
    }

    #[test]
    fn test_build_url_with_encoded_branch_template() {
        let (_dir, config) = local_config(
            r#"
[gitweb "host.xz"]
    branch-template = https://{host}/browse?at=refs%2Fheads%2F{branch_encoded}
"#,
        );

        let url = build_url(
            &branch("release-1.2_x~y/fix #3"),
            "git@host.xz:org/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok("https://host.xz/browse?at=refs%2Fheads%2Frelease-1.2_x~y%2Ffix%20%233".to_string())
        );
    }

    #[test]
    fn test_build_url_with_tag_template() {
        let (_dir, config) = local_config(
            r#"
[gitweb "host.xz"]
    branch-template = https://{host}/{repo}/branches/{branch}
    tag-template = https://{host}/{repo}/tags/{tag}?at={tag_encoded}
"#,
        );

        let url = build_url(&tag("release/1.0"), "git@host.xz:org/repo.git", &config);

        assert_eq!(
            url,
            Ok("https://host.xz/repo/tags/release/1.0?at=release%2F1.0".to_string())
        );
    }

    #[test]
    fn test_build_url_with_commit_template() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    commit-template = https://{host}/{repo}/commits/{commit}\n",
        );

        let url = build_url(&commit("abc123"), "git@host.xz:org/repo.git", &config);

        assert_eq!(url, Ok("https://host.xz/repo/commits/abc123".to_string()));
    }

    #[test]
    fn test_build_url_with_unknown_host_merge_request_template() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    merge-request-template = https://{host}/{repository}/pull-requests\n",
        );

        let url = build_url(&Target::MergeRequest, "git@host.xz:org/repo.git", &config);

        assert_eq!(
            url,
            Ok("https://host.xz/org/repo/pull-requests".to_string())
        );
    }

    #[test]
    fn test_build_url_without_template_for_the_target() {
        let (_dir, config) = local_config(BITBUCKET_SERVER_CONFIG);

        let url = build_url(
            &commit("abc123"),
            "git@bitbucket.company.com:PROJ/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok("https://bitbucket.company.com/PROJ/repo/commit/abc123".to_string())
        );
    }

    #[test]
    fn test_build_url_with_template_over_builtin_provider() {
        let (_dir, config) = local_config(
            "[gitweb \"github.com\"]\n    branch-template = https://{host}/{repository}/blob/{branch}/README.md\n",
        );

        let url = build_url(&branch("main"), "git@github.com:org/repo.git", &config);

        assert_eq!(
            url,
            Ok("https://github.com/org/repo/blob/main/README.md".to_string())
        );
    }

    #[test]
    fn test_build_url_with_unknown_placeholder() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    branch-template = https://{host}/{reop}/{branch}\n",
        );

        let url = build_url(&branch("main"), "git@host.xz:org/repo.git", &config);

        assert_eq!(
            url,
            Err(Issue::InvalidTemplate {
                key: "gitweb.host.xz.branch-template".to_string(),
                placeholder: "reop".to_string(),
                available: "{host}, {repository}, {owner}, {repo}, {branch}, {branch_encoded}"
                    .to_string(),
            })
        );
    }

    #[test]
    fn test_build_url_with_escaped_brace() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    branch-template = https://{host}/{{repo}/{branch}}\n",
        );

        let url = build_url(&branch("main"), "git@host.xz:org/repo.git", &config);

        assert_eq!(url, Ok("https://host.xz/{repo}/main}".to_string()));
    }

    #[test]
    fn test_build_url_with_placeholder_of_another_target() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    branch-template = https://{host}/{repo}/{commit}\n",
        );

        let url = build_url(&branch("main"), "git@host.xz:org/repo.git", &config);

        assert_eq!(
            url,
            Err(Issue::InvalidTemplate {
                key: "gitweb.host.xz.branch-template".to_string(),
                placeholder: "commit".to_string(),
                available: "{host}, {repository}, {owner}, {repo}, {branch}, {branch_encoded}"
                    .to_string(),
            })
        );
    }

    #[test]
    fn test_build_url_with_owner_of_repository_without_owner() {
        let (_dir, config) = local_config(
            "[gitweb \"host.xz\"]\n    branch-template = https://{host}/{owner}/{repo}\n",
        );

        let url = build_url(&branch("main"), "ssh://git@host.xz/repo.git", &config);

        assert_eq!(
            url,
            Err(Issue::InvalidTemplate {
                key: "gitweb.host.xz.branch-template".to_string(),
                placeholder: "owner".to_string(),
                available: "{host}, {repository}, {repo}, {branch}, {branch_encoded}".to_string(),
            })
        );
    }

    fn target_of(args: &[&str]) -> Target {
        let opt = Opt::try_parse_from(["gitweb"].iter().chain(args)).unwrap();

        Target::from_options(&opt, || "current".to_string())
    }

    #[test]
    fn test_target_from_options() {
        let cases = [
            (vec![], branch("current")),
            (vec!["--branch", "dev"], branch("dev")),
            (vec!["--tag", "v1"], tag("v1")),
            (vec!["--branch", "dev", "--tag", "v1"], tag("v1")),
            (vec!["--tag", "v1", "--commit", "abc123"], commit("abc123")),
            (vec!["--merge-request"], Target::MergeRequest),
        ];

        for (args, expected) in cases {
            assert_eq!(target_of(&args), expected, "for {args:?}");
        }
    }

    #[test]
    fn test_invalid_template_issue() {
        let issue = Issue::InvalidTemplate {
            key: "gitweb.host.xz.branch-template".to_string(),
            placeholder: "reop".to_string(),
            available: "{host}, {repo}".to_string(),
        };

        assert_eq!(
            issue.to_string(),
            "Unable to fill {reop} in gitweb.host.xz.branch-template, the available placeholders are {host}, {repo}"
        );
        assert_eq!(issue.exit_code(), 8);
    }

    #[test]
    fn test_build_url_with_github_tag() {
        let url = build_builtin_url(&tag("v1.0"), "git@github.com:yoannfleurydev/gitweb.git");

        assert_eq!(
            url,
            Ok("https://github.com/yoannfleurydev/gitweb/tree/v1.0".to_string())
        );
    }

    #[test]
    fn test_build_url_with_repository_template() {
        let (_dir, config) = local_config(
            r#"
[gitweb "gitlab.company.com"]
    branch-template = https://{host}/{repository}/-/tree/{branch}
"#,
        );

        let url = build_url(
            &branch("main"),
            "git@gitlab.company.com:group/subgroup/repo.git",
            &config,
        );

        assert_eq!(
            url,
            Ok("https://gitlab.company.com/group/subgroup/repo/-/tree/main".to_string())
        );
    }

    #[test]
    fn test_build_url_with_local_template_over_global_one() {
        let global = "[gitweb \"host.xz\"]\n    branch-template = https://global/{branch}\n";
        let local = "[gitweb \"host.xz\"]\n    branch-template = https://local/{branch}\n";

        // The local file is added first to check that the level wins, not the order.
        for files in [
            [(ConfigLevel::Local, local), (ConfigLevel::Global, global)],
            [(ConfigLevel::Global, global), (ConfigLevel::Local, local)],
        ] {
            let (_dir, config) = leveled_config(&files);

            let url = build_url(&branch("main"), "git@host.xz:org/repo.git", &config);

            assert_eq!(url, Ok("https://local/main".to_string()));
        }
    }

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
    fn test_with_http_dash_and_port_git_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("http://host-dash.xz:80/path/to/repo.git/").unwrap();

        assert_eq!(domain, "host-dash.xz");
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
    fn test_with_git_inside_the_repository_name_url_parts() {
        let RemoteParts { domain, repository } =
            get_remote_parts("git@github.com:yoannfleurydev/yoannfleurydev.github.io.git").unwrap();

        assert_eq!(domain, "github.com");
        assert_eq!(repository, "yoannfleurydev/yoannfleurydev.github.io");
    }

    #[test]
    fn test_get_merge_request_parts_with_github() {
        let MergeRequestParts { path, tail } = get_merge_request_parts(GITHUB_HOSTNAME).unwrap();

        assert_eq!(path, "pulls");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_gitlab() {
        let MergeRequestParts { path, tail } = get_merge_request_parts(GITLAB_HOSTNAME).unwrap();

        assert_eq!(path, "-/merge_requests");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_bitbucket() {
        let MergeRequestParts { path, tail } = get_merge_request_parts(BITBUCKET_HOSTNAME).unwrap();

        assert_eq!(path, "pull-requests");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_gitea() {
        let MergeRequestParts { path, tail } = get_merge_request_parts(GITEA_HOSTNAME).unwrap();

        assert_eq!(path, "pulls");
        assert_eq!(tail, "");
    }

    #[test]
    fn test_get_merge_request_parts_with_unknown_provider() {
        let result = get_merge_request_parts("host.xz");

        assert_eq!(result.err(), Some(Issue::UnknownProvider));
    }
}
