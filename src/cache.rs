use std::{path::Path, collections::HashMap, fs, time::Instant};

use crate::util::mkdirp;
use colored::Colorize;

const CACHE_DIR: &'static str = ".regit";
const TEMP_DIR: &'static str = ".tmp";
const CONFIG_FILE: &'static str = "config.rt";

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

        for line in fs::read_to_string(&cfg_path).unwrap().lines() {
            if line.is_empty() { continue; }

            let cols = line.split("\t").collect::<Vec<&str>>();
            let hash = *cols.get(0).unwrap();
            let dir = *cols.get(1).unwrap();

            log!(format!("Caching: {} {}", hash, dir));
            self.tree.insert(hash.into(), dir.into());
        }
        self
    }

    pub fn repair(mut self) -> Self {
        info!("Repairing cache directory...");
        success!("Directory fixed!");
        self
    }

    pub fn update(&mut self,  hash: &str, dir: &str) {
        info!("Updating cache...");
        let cached_hash = self.tree.get(hash);
        success!("Updated");
        self.tree.insert(hash.into(), dir.into());
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

        let absolute_cache_dir = path::absolute(format!(
            "{}/{}", dirs::home_dir().unwrap().to_str().unwrap(), CACHE_DIR)
        ).unwrap(); 
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