use crate::Issue;
use anyhow::Result;
use git2::{ErrorCode, Repository};

/// Get the current repository.
// Will check that the user is in a git repository.
pub fn get_repo() -> Result<Repository, Issue> {
    const CURRENT_WORKING_DIRECTORY: &str = ".";

    let repo =
        Repository::discover(CURRENT_WORKING_DIRECTORY).map_err(|_| Issue::NotInAGitRepository)?;

    Ok(repo)
}

// Get the current branch or return master.
pub fn get_branch(repo: &Repository) -> String {
    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => {
            error!("failed to get head ref '{}'", e);
            None
        }
    };

    let head = head.as_ref().and_then(|h| h.shorthand());
    debug!(
        "On branch '{}'",
        head.unwrap_or("Not currently on any branch")
    );

    String::from(head.unwrap_or("master"))
}
