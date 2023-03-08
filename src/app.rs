#![allow(dead_code)]

use crate::{
    options::ValidModes,
    repository::Repository, cache::Cache
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
            cache: Cache::new(),
            options,
        }
    }

    pub fn clone() {
        
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

    fn clone_with_tar(&self, dest: &str) {

    }
}
