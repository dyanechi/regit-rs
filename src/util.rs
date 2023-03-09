use std::{fs, path::{self, Path, PathBuf}, io, borrow::Cow, process::Command};

use super::*;
use async_recursion::async_recursion;
use url::Url;

// pub const BASE_DIR: &'static str = ".degit-rs";
// pub const TMP_DIR_NAME: &'static str = "tmp";
// pub const DEGIT_CONFIG_NAME: &'static str = "degit.json";

// pub fn base_dir() -> PathBuf {
//     let home = String::from_utf8(
//     Command::new("pwd")
//             .output()
//             .expect("should return user's directory")
//             .stdout
//     ).unwrap();
//     let base = PathBuf::new().join(format!("{}/{}", home, BASE_DIR).as_str()).to_owned();
//     mkdirp(&base);
//     base
// }

pub fn mkdirp(dir: &Path) {
    let dir = path::absolute(dir).unwrap();
    fs::create_dir_all(&dir).expect("should create all directories");
    // let parent = dir.parent().unwrap();
    // if parent == dir { return };

    // if !parent.exists() {
    //     mkdirp(&parent);
    // } else {
    //     if dir.exists() { return; };
        
    //     let dir = dir.to_str().unwrap_or_default().to_owned();
    //     println!("Creating directory: '{}'", dir);
    //     fs::create_dir(&dir).expect(&format!("failed creating directory '{}'", dir));
    // }
}

#[derive(Default)]
struct Agent {}
impl Agent {
    pub fn new(proxy: &str) -> Self {
        Self {  }
    }
}

struct FetchOptions {
    url: Url,
    hostname: Option<String>,
    path: Option<String>,
    agent: Option<Agent>,
}
impl FetchOptions {
    fn new(
        url: &str,
        agent: Option<Agent>,
    ) -> Self {
        let url = Url::parse(url).ok().unwrap();
        let hostname = Some(url.host_str().unwrap().to_string());
        let path = Some(url.path().to_string());
        FetchOptions { url, hostname, path, agent }
    }
}

impl Default for FetchOptions {
    fn default() -> Self {
        FetchOptions::new("", None)
    }
}

#[async_recursion]
pub async fn fetch(url: &str, dest: &str, proxy: &str) -> Result<(), String> {
    // let mut options = FetchOptions::new(url, None);

    // if proxy.len() > 0 {
    //     options.agent = Some(Agent::new(proxy));
    // }

    info!(format!("Fetching repository from '{}'", url));
    let res = minreq::get(url).send().unwrap();
    let code = res.status_code;
    if code >= 400 {
        return Err(format!("response failed: '{}'", code));
    } else if code >= 300 {
        fetch(res.headers.get("location").unwrap(), dest, proxy).await.unwrap();
    } else {
        info!(format!("Saving file to '{}'", dest));
        mkdirp(Path::new(dest).parent().unwrap());
        fs::write(dest, res.as_bytes()).expect("failed writing to file");
    }

    Ok(())
}

// pub fn copy_dir_sync(from: &Path, to: &Path) -> io::Result<()> {
//     // let (from, to) = (Path::new(from), Path::new(to));
//     if from.is_dir() {
//         let dir = fs::create_dir_all(to)?;
//         fs::read_dir(from).unwrap().for_each(|r| {
//             let entry = r.unwrap();
//             let path = entry.path();

//             if path.is_dir() { copy_dir_sync(&path, to); }
//             else {
//                 let target = Path::new(&to).join(entry.file_name());
//                 fs::copy(from, target);
//             }
//         })
//     }
//     Ok(())
// }

// pub fn assert_dir_exists(dest: &Path) {
//     if ! path_exists(dest) {
//         panic!("DIR_NOT_EXIST: directory '{}' does not exist", dest.to_str().unwrap_or_default());
//     };
// }

// pub fn assert_dir_is_empty(dest: &Path) {
//     if ! dir_is_empty(dest) {
//         panic!("DIR_NOT_EMPTY: directory '{}' contain files", dest.to_str().unwrap_or_default());
//     };
// }

// pub fn path_exists(dest: &Path) -> bool {
//     fs::try_exists(path::absolute(dest).unwrap()).unwrap()
// }

// pub fn dir_exists(dest: &Path) -> bool {
//     if path_exists(dest) {
//         return fs::metadata(dest).unwrap().is_dir()
//     }
//     false
// }

// pub fn file_exists(dest: &Path) -> bool {
//     if path_exists(dest) {
//         return fs::metadata(dest).unwrap().is_file()
//     }
//     false
// }

// pub fn dir_is_empty(dest: &Path) -> bool {
//     // assert_dir_exists(dest);
//     mkdirp(dest);
//     fs::read_dir(dest).unwrap().count() == 0
// }

// pub fn path_metadata(dest: &Path) -> std::io::Result<fs::Metadata> {
//     if path_exists(dest) {
//         return fs::metadata(dest)
//     }
//     Err(io::Error::new(io::ErrorKind::NotFound, "path doesn't exist"))
// }
// pub fn resolve_path(path: &Path) {
//     path::ab
// }

// pub fn stash_files(dir: &Path, dest: &Path) {
//     let tmp_dir = Path::new(dir).join(TMP_DIR_NAME);
//     fs::remove_dir_all(&tmp_dir).unwrap();
//     mkdirp(&tmp_dir);

//     fs::read_dir(&tmp_dir).unwrap().for_each(|r| {
//         let entry = r.unwrap();
//         let file_path = Path::new(dest).join(entry.path());
//         let target_path = Path::new(&tmp_dir).join(entry.path());
//         if file_path.is_dir() {
//             copy_dir_sync(&file_path, &target_path).unwrap();
//             fs::remove_dir_all(&file_path).unwrap();
//         } else {
//             fs::copy(&file_path, &target_path).unwrap();
//         }
//     })
// }

// pub fn unstash_files(dir: &Path, dest: &Path) {
//     let tmp_dir = Path::new(dir).join(TMP_DIR_NAME);
//     fs::remove_dir_all(&tmp_dir).unwrap();
//     mkdirp(&tmp_dir);

//     fs::read_dir(&tmp_dir).unwrap().for_each(|r| {
//         let entry = r.unwrap();
//         let file_path = Path::new(dest).join(entry.path());
//         let target_path = Path::new(&tmp_dir).join(entry.path());
//         if file_path.is_dir() {
//             copy_dir_sync(&file_path, &target_path).unwrap();
//             fs::remove_dir_all(&file_path).unwrap();
//         } else {
//             fs::copy(&file_path, &target_path).unwrap();
//         }
//     })
// }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e() {
        
    }
}