use std::{path::{Path, self, PathBuf}, fs, collections::HashMap, io, io::prelude::*, process::Stdio, hash::Hash};
use regex::Regex;
use tokio::{io::BufReader, process::Command};
use crate::util::{DEGIT_CONFIG_NAME, base_dir, unstash_files, self, file_exists, mkdirp, fetch, dir_exists};

const SUPPORTED: [&'static str; 4] = ["github", "gitlab", "bitbucket", "git.sr.ht"];
const RE_VALID_REPO: &'static str = r"^(?:(?:https://)?([^:/]+\.[^:/]+)/|git@([^:/]+)[:/]|([^/]+):)?([^/\s]+)/([^/\s#]+)(?:((?:/[^/\s#]+)+))?(?:/)?(?:#(.+))?";

#[derive(Default, Debug, Clone, Copy)]
enum ValidModes {
    #[default]
    Tar,
    Git,
}

#[derive(Default, Debug, Clone)]
struct Repository {
    url: String,
    site: String,
    user: String,
    name: String,
    sub_dir: String,
    _ref: String,
    ssh: String,
    mode: ValidModes,
}

impl Repository {
    pub fn parse(src: &str) -> Repository {
        let re = Regex::new(RE_VALID_REPO).expect("regex should be created");
    
        let matches = re.captures(src).unwrap();
        assert!(matches.len() >= 2, "BAD_SRC: could not parse '{}'", src);
    
        println!("{:?}", matches);
        let mut site = "github";
        for i in 1..=3 {
            if let Some(m) = matches.get(i) {
                site = m.as_str()
            }
        }
    
        let site = site.replace(r".(com|org)?[^:.]+$", "");
        println!("INFO: site is '{}'", site);
        if ! SUPPORTED.contains(&site.as_str()) {
            panic!("UNSUPPORTED_HOST: degit-rs supports GitHub, GitLab, SourceHut and BitBucket");
        }
    
        let user = matches.get(4).map_or("", |m| m.as_str()).to_string();
        let name = matches.get(5).map_or("", |m| m.as_str()).replace(r".git$/", "");
        let sub_dir = matches.get(6).map_or("", |m| m.as_str()).to_string();
        let _ref = matches.get(7).map_or("HEAD", |m| m.as_str()).to_string();
    
        let ext = match site.as_str() {
            "bitbucket" => "org",
            "git.sr.ht" => "",
            _ => "com"
        };
        let domain = format!("{}.{}", site, ext);
    
        let url = format!("https://{domain}/{user}/{name}");
        let ssh = format!("git@{domain}:{user}/{name}");
    
        let mode = {
            if SUPPORTED.contains(&site.as_str()) {
                ValidModes::Tar
            } else {
                ValidModes::Git
            }
        };
    
        Repository {
            site,
            user,
            name,
            _ref,
            url,
            ssh,
            sub_dir,
            mode
        }
    
    }
}

#[derive(Default, Debug)]
struct Ref {
    _type: String,
    name: String,
    hash: String,
}
impl Ref {
    pub fn new(_type: String, name: String, hash: String) -> Self {
        Ref { _type, name, hash }
    }
}
    

#[derive(Default, Debug)]
struct Action {
    files: Vec<PathBuf>
}
impl Action {
    pub fn new(files: Vec<PathBuf>) -> Self {
        Self { files }
    }
}

#[derive(Default, Debug)]
pub struct Degit {
    src: String,
    cache: bool,
    force: bool,
    verbose: bool,
    proxy: String,
    repo: Repository,
    mode: ValidModes,
    _has_stashed: bool,
}

impl Degit {
    pub fn new(src: &str) -> DegitBuilder {
        DegitBuilder::new(src)
    }

    fn _get_directives(&self, dest: &Path) -> PathBuf {
        Path::new(dest).join(DEGIT_CONFIG_NAME).to_owned()
    }

    fn _dir_is_empty(&self, dest: &Path) -> bool {
        if util::dir_is_empty(dest) {
            if self.force {
                eprintln!("DEST_NOT_EMPTY: directory '{}' is not empty. Flag 'force' active, continuing...", dest.to_str().unwrap_or_default());
                return false
            } else {
                eprintln!("DEST_NOT_EMPTY: directory '{}' is not empty. Use flag 'force' to override", dest.to_str().unwrap_or_default());
            }
        }
        println!("DEST_IS_EMPTY: directory '{}' is empty, continuing...", dest.to_str().unwrap_or_default());
        true
    }

    async fn _get_hash(repo: &Repository, cached: &HashMap<String, String>) -> String {
        let refs = fetch_refs(&repo).await;
        if repo._ref == "HEAD" {
            return refs.iter().find(|r| r._type == "HEAD").expect("should find hash of HEAD ref").hash.to_owned()
        }
        Self::_select_ref(refs, repo._ref.to_owned())
        // "".into()
    }

    fn _get_hash_from_cache(repo: &Repository, mut cached: HashMap<String, String>) -> String {
        if cached.contains_key(repo._ref.as_str()) {
            let hash = cached.get(repo._ref.as_str()).expect("should contain cached hash");
            println!("USING_CACHE: using cached commit hash {}", hash);
            return hash.to_owned();
        }
        eprintln!("CACHE_NOT_FOUND: hash doesn't exist in cache");
        "".to_owned()
    }

    fn _select_ref(refs: Vec<Ref>, selector: String) -> String {
        for _ref in &refs {
            if _ref.name == selector {
                println!("FOUND_MATCH: found matching commit hash: {}", _ref.hash);
                return _ref.hash.to_owned();
            }
        }

        if selector.len() < 8 {
            panic!("BAD_INPUT: selector must be longer than 8 characters");
        }

        for _ref in refs {
            if _ref.hash.starts_with(selector.as_str()) {
                println!("FOUND_MATCH: found commit hash starting with '{}': {}", selector, _ref.hash);
                return _ref.hash.to_owned()
            };
        }

        eprintln!("NOT_FOUND: failed to find matching hash with selector '{}'", selector);
        "".into()
    }

    async fn _clone_with_tar(&self, dir: &Path, dest: &Path) {
        let repo = self.repo.to_owned();
        let cached = HashMap::new();

        let hash = Self::_get_hash(&repo, &cached).await;

        let sub_dir = {
            if repo.sub_dir.is_empty() {
                format!("{}-{}{}", repo.name, hash, repo.sub_dir)
            } else { repo.sub_dir.replace("/", "") }
        };

        if hash.is_empty() {
            panic!("MISSING_REF: could not find commit hash for {}", repo._ref);
        }

        let file = Path::new(format!("{}/{}.tar.gz", dir.to_str().unwrap_or_default(), hash).as_str()).to_owned();
        let url = match repo.site.as_str() {
            "gitlab" => format!("{}/repository/archive.tar.gz?ref={}", repo.url, hash),
            "bitbucket" => format!("{}/get/{}.tar.gz", repo.url, hash),
            _ => format!("{}/archive/{}.tar.gz", repo.url, hash),
        };

        if !self.cache {
            if file_exists(dest) {
                eprintln!("FILE_EXISTS: '{}' already exists locally", file.stringify());
            } else {
                mkdirp(file.parent().unwrap());

                // if self.proxy.len() > 0 {
                    println!("PROXY: using proxy '{}'", self.proxy);
                    println!("DOWNLOADING: downloading {} to {}", url, file.stringify());
                    fetch(url.as_str(), &file.stringify(), &self.proxy).await.unwrap();

                    println!("DOWNLOADED");
                // }
            }
        }

        update_cache(dir, &repo, hash, cached);

        println!(
            "EXTRACTING: extracting {} from {} to {}",
            sub_dir,
            file.to_str().unwrap_or_default(),
            dest.to_str().unwrap_or_default()
        );

        mkdirp(dest);
        untar(&file, &dest, &sub_dir).await;
    }

    async fn _clone_with_git(&self, dir: &Path, dest: &Path) {
        println!("STATUS: cloning with git from {} to {}", self.src, dest.to_str().unwrap_or_default());
        Command::new("git")
            .arg("clone")
            .arg(&self.repo.url)
            .arg(dest)
            .spawn()
            .expect("executing git clone command should succeed");

        let file = path::absolute(Path::new(dest).join(".git")).unwrap();
        println!("STATUS: removing from {}", file.to_str().unwrap_or_default());
        Command::new("rm")
            .arg("-rf")
            .arg(&file)
            .spawn()
            .expect("executing git clone command should succeed");
    }

    pub async fn clone(&self, dest: &Path) {
        let Repository {
            site,
            user,
            name,
            _ref, ..
        } = &self.repo;

        let base = base_dir();

        let dir = Path::new(".degit-rs-tmp")
            .join(site)
            .join(user)
            .join(name);

        if !dir_exists(&dir) {
            mkdirp(&dir);
        }
        self._dir_is_empty(&dest); 

        if let ValidModes::Tar = self.mode {
            self._clone_with_tar(&dir, dest).await;
        } else {
            self._clone_with_git(&dir, dest).await;
        }

        println!("SUCCESS: cloned {}/{} #{} -> {}", user, name, _ref, dest.to_str().unwrap_or(""));

        // let directives = self._get_directives(dest);
        // for d in directives {

        // }
        // if self._has_stashed { unstash_files(&dir, dest) }
    }

    fn remove (dir: &Path, dest: &Path, action: Action) {
        let files = action.files;
        let mut removed_files: Vec<PathBuf> = vec![];

        files.iter().for_each(|file| {
            let file_path = Path::new(dest).join(file);
            if fs::try_exists(&file_path).unwrap() {
                if file_path.is_dir() {
                    fs::remove_dir_all(&file_path).unwrap();
                    removed_files.push(file_path.join("/"));
                } else {
                    removed_files.push(file_path);
                }
            } else {
                eprintln!("FILE_DOES_NOT_EXIST: {}", file_path.to_str().unwrap_or_default());
            }
        });

        if removed_files.len() > 0 {
            println!("REMOVED: {:?}", removed_files)
        }
    }
}

#[derive(Default)]
pub struct DegitBuilder {
    degit: Degit
}
impl DegitBuilder {
    fn new(src: &str) -> Self {
        let mut degit = Degit::default();
        degit.src = src.into();
        degit.repo = Repository::parse(src);

        println!("DegitBuilder: built new Repository:\n{:#?}", degit.repo);
        Self { degit }
    }

    pub fn with_cache(mut self, value: bool) -> Self { self.degit.cache = value; self }
    pub fn with_force(mut self, value: bool) -> Self { self.degit.force = value; self }
    pub fn with_verbose(mut self, value: bool) -> Self { self.degit.verbose = value; self }
    pub fn with_proxy(mut self, value: &str) -> Self { self.degit.proxy = value.into(); self }
    fn with_repo(mut self, value: Repository) -> Self { self.degit.repo = value; self }
    fn with_mode(mut self, value: ValidModes) -> Self { self.degit.mode = value; self }

    pub fn build(self) -> Degit {
        self.degit
    }
}

trait Stringify {
    fn stringify(&self) -> String;
}

impl Stringify for Path {
    fn stringify(&self) -> String {
        self.to_str().unwrap().to_owned()
    }
}

async fn untar(file: &Path, dest: &Path, sub_dir: &str) {
    let target = format!("{}/{}", dest.stringify(), sub_dir);
    println!("UNTAR: extracting '{}' to '{}'", file.stringify(), target);

    let file = fs::File::open(file).expect("should open file");
    let stream = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(stream);
    
    if sub_dir.is_empty() {
        println!("ARCHIVE_FULL_UNTAR: unpacking full archive to destination");
        archive.unpack(&dest).expect("should unpack tar to destination");
    } else {
        println!("ARCHIVE_PART_UNTAR: unpacking directory '{}' from archive to destination", sub_dir);
        // let lookup = format!("{}", sub_dir.stringify());
        for entry in archive.entries().expect("should unwrap archive") {
            let mut e = entry.expect("should retrieve entry from archive");
            let path = e.path().unwrap().stringify();
            let arr = path.split("/").collect::<Vec<&str>>();
            if arr.len() < 3 { continue; }
            let item = arr.get(1).unwrap();
            if item.eq(&sub_dir) {
                let mut rest = String::new();
                for i in 2..arr.len() {  rest.push('/'); rest.push_str(arr.get(i).unwrap()); }
                
                let target = format!("{}{}", target, rest);
                println!("Entry: {:?}\n", path);
                e.unpack(&target).expect("UNTAR_FAILED: should extract to destination");
            }
        }
    }
    // tar::Archive::new()
        // .unpack(&target).expect("should unpack tar to destination");
}

async fn fetch_refs(repo: &Repository) -> Vec<Ref> {
    let command = Command::new("git")
        .arg("ls-remote")
        .arg(repo.url.as_str())
        .stdout(Stdio::piped())
        .output()
        .await
        .expect("git command failed"); 

    let stdout = String::from_utf8(command.stdout).unwrap();

    let mut refs: Vec<Ref> = vec![];

    for filtered in stdout.as_str().split("\n").filter(|&x| !x.is_empty()) {
        let row = filtered as &str;
        let splitted = row.split("\t").collect::<Vec<&str>>();
        let (hash, _ref) = (splitted.get(0).unwrap_or(&""), splitted.get(1).unwrap_or(&""));
    
        println!("!DEBUG: hash: {}\t_ref: {}", hash, _ref);
        
        if *_ref == "HEAD" {
            refs.push(Ref::new("HEAD".into(), "".into(), hash.to_owned().into()));
            return refs;
        }

        let cmd = Command::new(_ref)
            .output()
            .await
            .expect("_ref command failed to execute")
            .stdout;

        let cmd_output = String::from_utf8(cmd)
            .expect("cmd_ouput to string should succeed");


        println!("!DEBUG: cmd_output: {}", cmd_output.as_str());

        let matches = 
            regex::Regex::new(r"/refs/\w+/(.+)/").unwrap()
            .captures(cmd_output.as_str());

        let (mut _type, mut name) = ("", "");
        if let Some(ms) = matches {
            if ms.len() == 0 { panic!("no matches found in set") }
            else {
                _type = ms.get(1).expect("matches[1] should exist").as_str();
                name = ms.get(2).expect("matches[2] should exist").as_str();
            }
        }

        if _type == "heads" { _type = "branch" }
        else if _type == "refs" { _type = "ref" }

        refs.push(Ref::new(_type.into(), name.into(), hash.to_owned().into()));
    }

    refs
}

fn update_cache(
    dir: &Path,
    repo: &Repository,
    hash: String,
    mut cached: HashMap<String, String>,
) {
    // let logs = 
    let dir = path::absolute(dir).unwrap();
    let old_hash = cached.get(&repo._ref).unwrap_or(&"".to_string()).to_owned();
    if old_hash == hash { return }
    else if old_hash.len() > 0 {
        let mut used = false;
        for val in cached.values() {
            if *val == hash {
                used = true; break;
            }
        }

        if !used {
            fs::remove_file(dir.join(format!("{}.tar.gz", old_hash)))
                .expect(format!("file should be deleted but failed").as_str());
        }
    }

    cached.insert(repo._ref.to_owned(), hash);

    let file_path = Path::new(&base_dir()).join("map.json");
    let file = fs::File::create(&file_path).expect(
        format!("should create a new file at '{}'", file_path.stringify()).as_str()
    );
    serde_json::to_writer(file, &cached).expect("should write hashmap to a file");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds() {
        let degit = Degit::new("path")
        .build();
    }
}