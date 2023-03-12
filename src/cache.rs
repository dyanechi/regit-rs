use std::{path::Path, collections::HashMap, fs};

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
        let cfg_path = Path::new(&self.dir).join(CONFIG_FILE);
        log!("Loading cache from config file", cfg_path.to_str().unwrap());

        if !cfg_path.exists() {
            warn!("Path doesn't exist, creating new file...");
            fs::File::create(&cfg_path).unwrap();
        }

        let file = fs::File::open(&cfg_path).expect("should open file");
        let reader = std::io::BufReader::new(file);
        let tree: CacheTree = serde_json::from_reader(reader).unwrap_or_default();
        for t in tree.values() {
            debug!("Retrieved cache for ref:", t);
        }
        self.tree = tree;
        self
    }

    pub fn repair(mut self) -> Self {
        info!("Repairing cache directory...");
        success!("Directory fixed!");
        self
    }

    pub fn update(&mut self, repo_ref: &str, hash: &str, repo_dir: &str) {
        info!("Updating cache...");
        if let Some(cached_hash) = self.tree.get(repo_ref) {
            if cached_hash == hash { return; }
            if !self.tree.values().collect::<Vec<_>>().contains(&&hash.to_string()) {
                std::fs::remove_dir(format!("{}/{}.tar.gz", repo_dir, hash)).unwrap();
            }
        }

        self.tree.insert(repo_ref.into(), hash.into());

        let cache_map_path = Path::new(self.dir()).join(CONFIG_FILE);
        let cache_file = {
            if !cache_map_path.exists() { std::fs::File::create(&cache_map_path).unwrap(); }
            std::fs::File::options().write(true).open(cache_map_path).unwrap()
        };
        serde_json::to_writer(cache_file, &self.tree).expect("should serialize config file");
        success!("Updated");
    }

    pub fn clean(&mut self) {
        warn!("Cleaning all cache files...");
        cmd!("rm", ["-rf", &self.dir]);
        success!("Cache is fresh and shiny ✨");
        self.tree.clear();
    }

    pub fn stash_files(&self, dir: &Path, dest: &Path) {

    }

    pub fn unstash_files(&self, dir: &Path, dest: &Path) {

    }

    pub(crate) fn dir(&self) -> &str { self.dir.as_ref() }
    pub(crate) fn tree(&self) -> &CacheTree { &self.tree }
    pub(crate) fn tree_mut(&mut self) -> &mut CacheTree { &mut self.tree }
}
impl Cache {
    // un
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