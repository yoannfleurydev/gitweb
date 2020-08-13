use crate::ExitCode;
use git2::{ErrorCode, Repository};
use anyhow::Result;
use std::process::exit;

/// Get the current repository.
// Will check that the user is in a git repository.
pub fn get_repo() -> Result<Repository> {
    const CURRENT_WORKING_DIRECTORY: &str = ".";

    Repository::discover(CURRENT_WORKING_DIRECTORY).map_err(|_| {
        debug!("Command failed, please run command inside a git directory");
        exit(ExitCode::NotInAGitRepository as i32);
    })
}

// Get the current branch or return master.
pub fn get_branch(repo: &Repository) -> String {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => panic!("failed to get head ref {}", e),
    };

    let head = head.as_ref().and_then(|h| h.shorthand());
    debug!(
        "# On branch {}",
        head.unwrap_or("Not currently on any branch")
    );

    String::from(head.unwrap_or("master"))
}
