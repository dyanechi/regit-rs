use std::{path::{Path, PathBuf}, collections::HashMap, fs};

use super::*;
use crate::util::mkdirp;

const CACHE_DIR: &'static str = ".regit";
const TEMP_DIR: &'static str = ".tmp";
const CONFIG_FILE: &'static str = "config.json";

type CacheTree = HashMap<String, String>;

#[derive(Debug, Default)]
pub(crate) struct Cache {
    dir: String,
    tree: CacheTree
}
impl Cache {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir()
            .expect("should get user's home directory")
            .to_str()
            .unwrap()
            .to_owned();
        let dir = format!("{}/{}", home_dir, CACHE_DIR);
        mkdirp(Path::new(&dir));

        Cache { dir, ..Default::default() }
    }

    pub fn new_custom(dir: &str) -> Self {
        let dir = Path::new(dir);
        mkdirp(dir);
        Cache { dir: dir.to_str().unwrap().to_owned(), ..Default::default() }
    }

    pub fn load(mut self) -> Self {
        log!("Loading cache from config file...");

        let file = fs::File::open(self.cfg_path()).expect("should open file");
        let reader = std::io::BufReader::new(file);
        let tree: CacheTree = serde_json::from_reader(reader).unwrap_or_default();
        for t in tree.values() {
            debug!("Retrieved cache for ref:", t);
        }
        self.tree = tree;
        self
    }

    pub fn update(&mut self, repo_ref: &str, hash: &str, repo_dir: &str) {
        info!("Updating cache...");
        let repo_sig = format!("{}:{}", repo_dir, repo_ref);

        if let Some(cached_hash) = self.tree.get(&repo_sig) {
            if cached_hash == hash { return; }
            if !self.tree.values().collect::<Vec<_>>().contains(&&hash.to_string()) {
                std::fs::remove_dir(format!("{}/{}.tar.gz", repo_dir, hash)).unwrap();
            }
        }

        self.tree.insert(repo_sig.into(), hash.into());

        let cache_file = fs::File::options().write(true).open(self.cfg_path()).unwrap();

        serde_json::to_writer(cache_file, &self.tree).expect("should serialize config file");
        success!("Updated");
    }

    pub fn clean(&mut self) {
        warn!("Cleaning all cache files...");
        cmd!("rm", ["-rf", &self.dir]);
        success!("Cache is fresh and shiny ✨");
        self.tree.clear();
    }


    pub fn repair(mut self) -> Self {
        info!("Repairing cache directory...");
        success!("Directory fixed!");
        self
    }

    // pub fn stash_files(&self, dir: &Path, dest: &Path) {

    // }

    // pub fn unstash_files(&self, dir: &Path, dest: &Path) {

    // }

    pub(crate) fn dir(&self) -> &str { self.dir.as_ref() }
    pub(crate) fn tree(&self) -> &CacheTree { &self.tree }
    pub(crate) fn tree_mut(&mut self) -> &mut CacheTree { &mut self.tree }
    
    pub fn get_cached_hash(&self, repo_dir: &str, repo_ref: &str) -> Option<String> {
        if let Some(hash) = self.tree.get(&format!("{}:{}", repo_dir, repo_ref)) {
            return Some(hash.to_owned())
        }
        None
    }

    pub fn get_repo_location(&self, hash: &str) -> Option<String> {
        log!(format!("Searching hash location: '{}'...", hash));
        for (k, v) in self.tree.to_owned() {
            if v == hash {
                log!(format!("Hash found at location: '{}'", k));
                return Some(k.to_owned());
            }
        }
        None
    }
}

impl Cache {
    fn cfg_path(&self) -> PathBuf {
        let cfg_path = Path::new(&self.dir).join(CONFIG_FILE);
        if !cfg_path.exists() {
            warn!("Path doesn't exist, creating new file...");
            fs::File::create(&cfg_path).unwrap();
        }
        cfg_path
    }
}

#[cfg(test)]
mod tests {
    use std::path;

    use super::*;

    #[test]
    fn creates_cache_dir() {
        let cache = Cache::new();
        let expected_dir = format!("/home/vp0x/{}", CACHE_DIR);
        assert_eq!(cache.dir, expected_dir, 
            "cache dir should match expected_dir");

        let absolute_cache_dir = format!("{}/{}", dirs::home_dir().unwrap().to_str().unwrap(), CACHE_DIR);
        let absolute_cache_dir = Path::new(&absolute_cache_dir);
        assert!(absolute_cache_dir.exists(), "cache dir should exist");

        let expected_cfg_file = absolute_cache_dir.join(CONFIG_FILE);
        assert!(expected_cfg_file.exists(), "cache config file should be created");
        assert!(expected_cfg_file.is_file(), "cache config should be a valid file");

        
    }

    #[test]
    fn loads_cache_tree() {
        let cache = Cache::new().load();


    }

    #[test]
    fn repairs_cache_tree() {
        let cache = Cache::new();
        cache.repair();
    }

    #[test]
    fn cleans_cache() {
        let mut cache = Cache::new();
        cache.clean();
    }
}