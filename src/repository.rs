use std::{collections::HashMap, process, str};

use super::*;
use regex::Regex;

use crate::{options::ValidModes, cache::Cache};

const SUPPORTED_DOMAINS: [&'static str; 4] = ["github.com", "gitlab.com", "bitbucket.org", "git.sr.ht"];
const RE_VALID_REPO: &'static str = r"^(?:(?:https://)?([^:/]+\.[^:/]+)/|git@([^:/]+)[:/]|([^/]+):)?([^/\s]+)/([^/\s#]+)(?:((?:/[^/\s#]+)+))?(?:/)?(?:#(.+))?";

type HashCache = HashMap<String, String>;

#[derive(Default, Debug, Clone)]
struct Ref {
    kind: String,
    name: String,
    hash: String,
}
impl Ref {
    pub fn new(
        kind: &str,
        name: &str,
        hash: &str,
    ) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            hash: hash.into()
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Repository {
    pub url: String,
    pub domain: String,
    pub user: String,
    pub name: String,
    pub sub_dir: String,
    pub _ref: String,
    pub ssh: String,
    pub mode: ValidModes,
    refs: Vec<Ref>
}
impl Repository {
    pub fn parse(src: &str) -> Self {
        log!(format!("Parsing repository: '{}'...", src));
        let re = Regex::new(RE_VALID_REPO).unwrap();
        let matches = re.captures(src).unwrap();

        let mut domain = String::from("github.com");
        for i in 1..=3 {
            if let Some(m) = matches.get(i) { domain = m.as_str().into(); }
        }

        let mut mode = ValidModes::Tar;
        if ! SUPPORTED_DOMAINS.contains(&domain.as_str()) {
            mode = ValidModes::Git;
            warn!("ReGit only supports GitHub, GitLab, SourceHut and BitBucket domains.");
            warn!("Switching to 'Git' mode. It might not work properly and will be slower.");
            // panic!("domain not supported");
        }

        let user = matches.get(4).map_or("", |m| m.as_str()).to_string();
        let name = matches.get(5).map_or("", |m| m.as_str()).to_string();
        let sub_dir = matches.get(6).map_or("", |m| m.as_str()).to_string();
        let _ref = matches.get(7).map_or("HEAD", |m| m.as_str()).to_string();

        let ssh = format!("git@{domain}:{user}/{name}");
        let url = format!("https://{domain}/{user}/{name}");

        let refs = Self::fetch_refs(url.as_str());

        success!(format!("Using '{}' as repository source", url));

        Repository { 
            url, 
            domain, 
            user, 
            name, 
            sub_dir, 
            _ref, 
            ssh, 
            mode, 
            refs,
        }
    }

    pub fn archive_url(&self, hash: &str) -> String {
        match self.domain.as_str() {
            "gitlab" => format!("{}/repository/archive.tar.gz?ref={}", self.url, hash),
            "bitbucket" => format!("{}/get/{}.tar.gz", self.url, hash),
            _ => format!("{}/archive/{}.tar.gz", self.url, hash),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}{}", self.user, self.name, self.sub_dir)
    }

    pub fn full_name_ref(&self) -> String {
        format!("{}:{}", self.full_name(), self._ref)
    }

    pub fn get_hash(&self) -> String {
        if self._ref == "HEAD" {
            return self.refs.iter().find(|_ref| _ref.kind == "HEAD")
                .expect("should find hash of HEAD ref")
                .hash.to_owned()
        }
        self._select_ref(self._ref.as_str()).hash
    }

    pub(crate) fn get_hash_cached(&self, cache: &Cache) -> String {
        if let Some(hash) = cache.get_cached_hash(&self.full_name(), &self._ref) {
            return hash;
        }
        self.get_hash()
    }
}
impl Repository {
    fn fetch_refs(url: &str) -> Vec<Ref> {
        log!("Fetching refs...");
        let output = cmd!("git", ["ls-remote", url]).stdout;
        let stdout = str::from_utf8(&output).unwrap();

        let mut refs = vec![];
        for row in stdout.split("\n").filter(|x| !x.is_empty()) {
            let cols = row.split("\t").collect::<Vec<&str>>();
            let hash = *cols.get(0).unwrap();
            let _ref = *cols.get(1).unwrap();

            if _ref == "HEAD" {
                return vec![Ref::new("HEAD", "", hash)];
            }

            let cmd_refs = String::from_utf8(cmd!(_ref).stdout).unwrap();
            let matches = regex::Regex::new(r"/refs/\w+/(.+)/").unwrap().captures(&cmd_refs);
            let (kind, name) = match matches {
                Some(ms) => {
                    if ms.len() < 3 { continue; }
                    (ms.get(1).unwrap().as_str(), ms.get(2).unwrap().as_str())
                },
                None => continue
            };

            refs.push(Ref::new(kind, name, hash));
        }

        println!("\nFetched refs: {:#?}\n", refs);
        refs
    }

    fn _select_ref(&self, selector: &str) -> Ref {
        println!("Selecting ref with selector '{}'...", selector);
        for r in &self.refs {
            if r.name == selector {
                println!("Found ref exact match '{}'", r.name);
                return r.to_owned();
            }
        }

        assert!(selector.len() >= 8, "selector must be at least 8 char long");
        for r in &self.refs {
            if r.hash.starts_with(selector) {
                println!("Found ref partial match '{}'", r.name);
                return r.to_owned();
            }
        }
        panic!("FATAL: ref with selector '{}' not found in current repository", selector);
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SRC: &'static str = "solidjs/templates/ts";

    #[test]
    fn parses_repository() {
        println!("Testing...");
        let repo = Repository::parse(TEST_SRC);

        assert_eq!(repo.url, "https://github.com/solidjs/templates");
        assert_eq!(repo.domain, "github.com");
        assert_eq!(repo.sub_dir, "ts");
        assert_eq!(repo.user, "solidjs");
        assert_eq!(repo.name, "templates");
        assert_eq!(repo.ssh, "git@github.com:solidjs/templates");
        assert_eq!(repo.mode, ValidModes::Tar);
        assert_eq!(repo._ref, "HEAD");
    }

    #[test]
    fn fetches_refs() {
        let repo = Repository::parse(TEST_SRC);

        assert!(!repo.refs.is_empty(), "refs are empty: {:#?}", repo.refs);
    }
}

impl Repository {
    pub fn url(&self) -> String { self.url.to_owned() }
}