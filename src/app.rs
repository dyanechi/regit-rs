#![allow(dead_code)]

use std::path::Path;

use super::*;
use crate::{
    options::ValidModes,
    repository::Repository, cache::Cache, util::{mkdirp, fetch}
};


#[derive(Debug, Default, Clone, Copy)]
pub struct RegitOptions {
    cache: bool,
    force: bool,
    verbose: bool,
    has_stashed: bool,
}

#[derive(Debug, Default)]
pub struct Regit {
    src: String,
    repo: Repository,
    cache: Cache,
    options: RegitOptions,
}

impl Regit {
    pub fn new(src: &str, options: RegitOptions) -> Self {
        Self {
            src: src.into(),
            repo: Repository::parse(src),
            cache: Cache::new().load(),
            options,
        }
    }

    pub async fn clone(&mut self, dest: &str) {
        let Repository {
            domain,
            user,
            name,
            _ref, ..
        } = &self.repo;

        let dir = Path::new(self.cache.dir())
            .join(format!("{}/{}/{}", domain, user, name));

        if ! dir.exists() { mkdirp(&dir) }
        match self.repo.mode {
            ValidModes::Tar => self.clone_with_tar(&dir, Path::new(dest)).await,
            ValidModes::Git => self.clone_with_git(dest)
        }
    }
}

impl Regit {
    fn clone_with_git(&self, dest: &str) {
        println!("Cloning with git to {}...", dest);
        cmd!("git", ["clone", &self.repo.url(), dest]);

        let git = format!("{}/.git", dest);
        println!("Removing .git directory from {}...", dest);
        cmd!("rm", ["-rf", &git]);
    }

    async fn clone_with_tar(&mut self, dir: &Path, dest: &Path) {
        info!("Cloning repository in Tar mode...");
        let repo = self.repo.to_owned();
        let cache = self.cache.tree_mut();
        let hash = repo.get_hash_cached(&cache);
        let archive_url = repo.archive_url(&hash);
        let sub_dir = if repo.sub_dir.is_empty() {
            format!("{}-{}", repo.name, hash)
        } else { repo.sub_dir.replacen("/", &repo.sub_dir, 1) };

        let file = Path::new(&format!("{}/{}", dir.to_str().unwrap(), hash)).to_owned();

        if !dest.exists() { mkdirp(dest) }
        if self.options.cache {
            if file.exists() && file.is_file() {
                info!("File found in cache! Using it to make things faster...");
                Self::untar(&file, &dest, &sub_dir);
                return;
            }
        }
        
        fetch(&archive_url, dest.to_str().unwrap(), "").await.unwrap();
        self.cache.update(&hash, dir.to_str().unwrap());
        Self::untar(&file, &dest, &sub_dir);

    }

    fn clone_from_cache(&self, dest: &str) {

    }
}

impl Regit {
    fn untar(file: &Path, dest: &Path, sub_dir: &str) {
        info!(format!(
            "Extracting '{}' from '{}' to '{}'",
            sub_dir,
            file.to_str().unwrap_or_default(),
            dest.to_str().unwrap_or_default()
        ));

        
    }
}
