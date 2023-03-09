#![allow(dead_code)]

use std::path::Path;

use super::*;
use crate::{
    options::ValidModes,
    repository::Repository, cache::Cache, util::{mkdirp, fetch}, traits::{AsStr, AsString}
};


#[derive(Debug, Clone, Copy)]
pub struct RegitOptions {
    cache: bool,
    force: bool,
    verbose: bool,
    has_stashed: bool,
}
impl Default for RegitOptions {
    fn default() -> Self {
        Self { 
            cache: true, 
            force: false, 
            verbose: false,
            has_stashed: false
        }
    }
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
        info!("Cloning repository...");
        let Repository {
            domain,
            user,
            name,
            _ref, ..
        } = &self.repo;

        let dest_path= std::path::absolute(dest).unwrap();
        if !dest_path.exists() {
            warn!(format!("'{}' doesn't exist! Attempting to create path...", dest_path.as_str()));
            mkdirp(&dest_path);
        }

        let repo_dir = Path::new(self.cache.dir())
            .join(format!("{}/{}/{}", domain, user, name));

        if ! repo_dir.exists() { mkdirp(&repo_dir) }
        match self.repo.mode {
            ValidModes::Tar => self.clone_with_tar(&repo_dir, &dest_path).await,
            ValidModes::Git => self.clone_with_git(dest)
        }

        done!("Repository successfully cloned. Happy coding!");
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

    async fn clone_with_tar(&mut self, repo_dir: &Path, dest: &Path) {
        info!("Cloning repository in Tar mode...");
        let repo = self.repo.to_owned();
        let cache = self.cache.tree_mut();
        let hash = repo.get_hash_cached(&cache);
        let archive_url = repo.archive_url(&hash);

        log!("Archive url is", &archive_url, "...");
        // let sub_dir = if repo.sub_dir.is_empty() {
        //     format!("{}-{}", repo.name, hash)
        // } else { repo.sub_dir.replacen("/", &repo.sub_dir, 1) };
        let sub_dir = format!("{}-{}/{}", repo.name, hash, repo.sub_dir);
        let file = Path::new(&format!("{}/{}.tar.gz", repo_dir.as_str(), hash)).to_owned();

        if !dest.exists() { mkdirp(dest) }
        if std::fs::read_dir(dest).unwrap().count() > 0 {
            error!(format!("Destination '{}' not empty!", dest.display())); panic!("dir not empty");
        }
        if file.exists() && file.is_file() {
            if self.options.cache {
                success!("File found in cache! Using it to make things faster...".dimmed());
                Self::untar(&file, &dest, &sub_dir);
                return;
            }
        } else { mkdirp(file.parent().unwrap()); }
        
        
        fetch(&archive_url, file.as_str(), "").await.unwrap();
        self.cache.update(&hash, &repo._ref, repo_dir.as_str());
        Self::untar(&file, &dest, &sub_dir);
    }

    fn clone_from_cache(&self, dest: &str) {

    }
}

impl Regit {
    fn untar(file: &Path, dest: &Path, sub_dir: &str) {
        let archive_name = file.file_prefix().unwrap().to_str().unwrap();
        // let target = format!("{}/{}", dest.as_str(), sub_dir);
        // info!(format!("Extracting '...{}' to '{}'", file.as_str(), dest.as_str()));


        let file = std::fs::File::open(file).expect("error opening file");
        let stream = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(stream);

        if sub_dir.is_empty() {
            debug!("Unpacking full archive to destination...");
            archive.unpack(dest).expect("unpacking directory failed");
        } else {
            let mut count_unpacked = 0usize;
            debug!(format!("Unpacking '{}' to destination...", sub_dir));
            for entry in archive.entries().unwrap() {
                let mut entry = entry.unwrap();
                let entry_path = entry.path().unwrap().as_string();
                let dir_path = format!("{}/", sub_dir);

                // log!(&entry_path);
                // debug!("Matching against:", &format!("{}/", sub_dir));
                if entry_path.starts_with(&dir_path) {
                    let untar_file = entry_path.replace(&dir_path, "");
                    let file_path = format!("{}/{}", dest.as_str(), untar_file);
                    log!(format!("Extracting '{}' to '{}'...", untar_file, file_path));
                    entry.unpack(&file_path).expect(" should extract to destination");
                    count_unpacked += 1;
                }
            };
            if count_unpacked == 0 { warn!("No files unpacked"); }
            else { success!(&format!("Unpacked {} files", count_unpacked)); }
        }
    }
}
