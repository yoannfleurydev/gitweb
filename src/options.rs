use clap::Parser;
use clap_complete::{ArgValueCandidates, CompletionCandidate};
use git2::Repository;

use crate::git;

#[derive(Debug, Parser)]
#[command(version, about)]
// Rename all will use the name of the field
#[command(rename_all = "kebab-case")]
pub struct Opt {
    /// Set the branch (alias for --tag)
    ///
    /// By setting the branch, you can override the default behavior that will
    /// set the branch to the current one in the repository. If something went
    /// wrong with the current one, it will set the value to master.
    #[arg(short, long, add = ArgValueCandidates::new(complete_branches))]
    pub branch: Option<String>,

    /// Set the tag (alias for --branch).
    ///
    /// By setting the tag, you can override the default behavior that will set
    /// the branch to the current one in the repository and will instead take
    /// the reference tag (or branch) given as parameter.
    #[arg(short, long, add = ArgValueCandidates::new(complete_tags))]
    pub tag: Option<String>,

    /// Set the merge request flag.
    ///
    /// By setting the merge_request flag, you can override the default behavior that will
    /// open up the merge requests listing page.
    #[arg(short = 'M', long = "merge-request", conflicts_with_all = ["commit", "tag", "branch"])]
    pub merge_request: bool,

    /// Set a commit
    ///
    /// By setting a commit, you can override the default behavior that will
    /// set the branch to the current one in the repository.
    #[arg(short, long)]
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
    #[arg(short = 'B', long, env = "BROWSER")]
    pub browser: Option<String>,

    /// Set the remote
    ///
    /// By default, the selected remote will be origin by convention. You can
    /// override this setting by  using this option.
    #[arg(short, long, add = ArgValueCandidates::new(complete_remotes))]
    pub remote: Option<String>,

    /// Set the verbosity of the command
    ///
    /// By settings this option, you will have more feedback on the output.
    #[arg(short, long)]
    pub verbose: bool,
}

fn complete_branches() -> Vec<CompletionCandidate> {
    complete_with(git::get_branches)
}

fn complete_tags() -> Vec<CompletionCandidate> {
    complete_with(git::get_tags)
}

fn complete_remotes() -> Vec<CompletionCandidate> {
    complete_with(git::get_remotes)
}

// Complete with the values from the current repository. Nothing is suggested
// outside of a repository, as anything printed would end up in the shell.
fn complete_with(get_values: fn(&Repository) -> Vec<String>) -> Vec<CompletionCandidate> {
    git::get_repo()
        .map(|repo| {
            get_values(&repo)
                .into_iter()
                .map(CompletionCandidate::new)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_options_are_valid() {
        Opt::command().debug_assert();
    }
}
