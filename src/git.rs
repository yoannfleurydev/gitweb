use crate::Issue;
use anyhow::Result;
use git2::{BranchType, ErrorCode, Repository};
use std::collections::BTreeSet;

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

    let head = head.as_ref().and_then(|h| h.shorthand().ok());
    debug!(
        "On branch '{}'",
        head.unwrap_or("Not currently on any branch")
    );

    String::from(head.unwrap_or("master"))
}

// Get the local and remote-tracking branch names, without the remote prefix.
pub fn get_branches(repo: &Repository) -> Vec<String> {
    let Ok(branches) = repo.branches(None) else {
        return Vec::new();
    };

    let remotes = get_remotes(repo);

    // A set removes the duplicates between local and remote branches and sorts
    // the names.
    let names: BTreeSet<String> = branches
        .flatten()
        .filter_map(|(branch, branch_type)| {
            let name = branch.name().ok().flatten()?;

            match branch_type {
                BranchType::Local => Some(name.to_string()),
                BranchType::Remote => remotes
                    .iter()
                    .find_map(|remote| name.strip_prefix(&format!("{}/", remote)))
                    .map(String::from),
            }
        })
        .filter(|name| name != "HEAD")
        .collect();

    names.into_iter().collect()
}

// Get the tag names.
pub fn get_tags(repo: &Repository) -> Vec<String> {
    repo.tag_names(None)
        .map(|tags| tags.iter().flatten().flatten().map(String::from).collect())
        .unwrap_or_default()
}

// Get the remote names.
pub fn get_remotes(repo: &Repository) -> Vec<String> {
    repo.remotes()
        .map(|remotes| {
            remotes
                .iter()
                .flatten()
                .flatten()
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{Commit, RepositoryInitOptions, Signature};
    use tempfile::TempDir;

    // Create a repository with a single commit on the main branch.
    fn init_repo() -> (TempDir, Repository) {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init_opts(
            dir.path(),
            RepositoryInitOptions::new().initial_head("main"),
        )
        .unwrap();

        {
            let signature = Signature::now("gitweb", "gitweb@example.com").unwrap();
            let tree_id = repo.index().unwrap().write_tree().unwrap();
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &signature, &signature, "init", &tree, &[])
                .unwrap();
        }

        (dir, repo)
    }

    fn head_commit(repo: &Repository) -> Commit<'_> {
        repo.head().unwrap().peel_to_commit().unwrap()
    }

    #[test]
    fn test_get_branches_merges_local_and_remote_branches() {
        let (_dir, repo) = init_repo();
        let commit = head_commit(&repo);

        repo.branch("local-only", &commit, false).unwrap();
        repo.remote("origin", "git@github.com:yoannfleurydev/gitweb.git")
            .unwrap();
        repo.reference("refs/remotes/origin/main", commit.id(), false, "")
            .unwrap();
        repo.reference("refs/remotes/origin/remote-only", commit.id(), false, "")
            .unwrap();
        repo.reference_symbolic(
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
            false,
            "",
        )
        .unwrap();

        assert_eq!(
            get_branches(&repo),
            vec!["local-only", "main", "remote-only"]
        );
    }

    #[test]
    fn test_get_branches_without_commit() {
        let dir = tempfile::tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        assert!(get_branches(&repo).is_empty());
    }

    #[test]
    fn test_get_tags() {
        let (_dir, repo) = init_repo();
        let commit = head_commit(&repo);

        repo.tag_lightweight("v0.1.0", commit.as_object(), false)
            .unwrap();
        repo.tag_lightweight("v0.2.0", commit.as_object(), false)
            .unwrap();

        let mut tags = get_tags(&repo);
        tags.sort();

        assert_eq!(tags, vec!["v0.1.0", "v0.2.0"]);
    }

    #[test]
    fn test_get_remotes() {
        let (_dir, repo) = init_repo();

        repo.remote("upstream", "git@github.com:yoannfleurydev/gitweb.git")
            .unwrap();
        repo.remote("origin", "git@github.com:someone/gitweb.git")
            .unwrap();

        assert_eq!(get_remotes(&repo), vec!["origin", "upstream"]);
    }
}
