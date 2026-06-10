// Git backend for tracking /etc changes

use anyhow::{Context, Result};
use git2::{Repository, Signature, Oid, ResetType};
use std::path::{Path, PathBuf};
use tracing::info;

pub struct GitStore {
    git_dir: PathBuf,
    work_dir: PathBuf,
    repo: Option<Repository>,
}

impl GitStore {
    pub fn new(git_dir: &Path, work_dir: &Path) -> Result<Self> {
        Ok(Self {
            git_dir: git_dir.to_path_buf(),
            work_dir: work_dir.to_path_buf(),
            repo: None,
        })
    }

    pub fn init(&mut self) -> Result<()> {
        if self.git_dir.join("HEAD").exists() {
            self.repo = Some(Repository::open_bare(&self.git_dir)
                .context("Failed to open existing git repo")?);
            info!("Opened existing git repository");
        } else {
            std::fs::create_dir_all(&self.git_dir)?;
            self.repo = Some(Repository::init_bare(&self.git_dir)
                .context("Failed to init bare git repo")?);
            info!("Initialized new git repository at {:?}", self.git_dir);
        }
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<Oid> {
        let repo = self.repo.as_ref()
            .context("Git repo not initialized")?;

        let signature = Signature::now("KairosOS Agent", "agent@kairosos.local")
            .context("Failed to create signature")?;

        // Create a tree from the working directory
        let mut index = repo.index()?;

        // Add all files from the work directory
        self.add_to_index(&self.work_dir, &mut index, "")?;

        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        // Get parent commit (if any)
        let parent = match repo.head() {
            Ok(head) => {
                let oid = head.target().context("No target")?;
                Some(repo.find_commit(oid)?)
            }
            Err(_) => None,
        };

        let parents: Vec<&git2::Commit> = parent.iter().collect();

        let commit_oid = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )?;

        Ok(commit_oid)
    }

    fn add_to_index(&self, dir: &Path, index: &mut git2::Index, prefix: &str) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            let rel_path = if prefix.is_empty() {
                name_str.to_string()
            } else {
                format!("{}/{}", prefix, name_str)
            };

            if path.is_dir() {
                // Skip .git directory
                if name_str != ".git" {
                    self.add_to_index(&path, index, &rel_path)?;
                }
            } else if path.is_file() {
                index.add_path(Path::new(&rel_path))?;
            }
        }

        Ok(())
    }

    pub fn log(&self, max_count: usize) -> Result<Vec<String>> {
        let repo = self.repo.as_ref()
            .context("Git repo not initialized")?;

        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut entries = Vec::new();

        for (i, oid) in revwalk.enumerate() {
            if i >= max_count {
                break;
            }
            let oid = oid?;
            let commit = repo.find_commit(oid)?;
            entries.push(format!(
                "{} {}",
                &oid.to_string()[..8],
                commit.message().unwrap_or("").lines().next().unwrap_or("")
            ));
        }

        Ok(entries)
    }
}
