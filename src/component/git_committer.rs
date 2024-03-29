use std::path::Path;

use git2;


pub struct GitCommitter {
    user_repo: git2::Repository
}

impl GitCommitter {
    pub fn new(user_repo: git2::Repository) -> Self {
        GitCommitter { user_repo: user_repo }
    }

    pub fn detect_and_commit(&mut self) -> Result<(), git2::Error> {
        let mut opts = git2::StatusOptions::new();
        opts.recurse_untracked_dirs(true);
        opts.include_untracked(true);

        for status_entry in self.user_repo.statuses(Some(&mut opts))?.iter() {
            let mut index = self.user_repo.index()?;
            let changed_file_path = Path::new(status_entry.path().unwrap());
            let changed_file_str = changed_file_path.to_str().unwrap();
            let author = "stas@localhost";
            
            let commit_message = match status_entry.status() {
                s if s.contains(git2::Status::WT_MODIFIED) => {
                    index.add_path(changed_file_path)?;
                    format!("File {path} modified by {author}", path = changed_file_str, author = author)
                }
                s if s.contains(git2::Status::WT_NEW) => {
                    index.add_path(changed_file_path)?;
                    format!("File {path} added by {author}", path = changed_file_str, author = author)
                }
                s if s.contains(git2::Status::WT_DELETED) => {
                    index.remove_path(changed_file_path)?;
                    format!("File {path} deleted by {author}", path = changed_file_str, author = author)
                }
                _ => continue,
            };

            let tree_id = index.write_tree()?;
            let tree = self.user_repo.find_tree(tree_id)?;
            let sig = self.user_repo.signature()?;
            let head_commit = self.user_repo.head()?.peel_to_commit()?;
            self.user_repo.commit(Some("HEAD"), &sig, &sig, &commit_message, &tree, &[&head_commit])?;
            index.write()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::testing::TestingGitRepo;
    use super::*;

    #[test]
    fn test_no_changes() {
        // given a clean git repo
        let testing_git_repo = TestingGitRepo::new();
        // and a git committer
        let mut git_committer = GitCommitter::new(git2::Repository::open(testing_git_repo.path()).unwrap());

        // when calling method detect_and_commit
        let git_status_before = testing_git_repo.status();
        git_committer.detect_and_commit().unwrap();
        let git_status_after = testing_git_repo.status();

        // then the result of `git status` doesn't change
        assert_eq!(git_status_before, git_status_after);
    }
}
