use std::{fs, path::{self, Path, PathBuf}, io, borrow::Cow, process::Command};

use super::*;
use async_recursion::async_recursion;
use url::Url;

pub fn mkdirp(dir: &Path) {
    let dir = path::absolute(dir).unwrap();
    fs::create_dir_all(&dir).expect("should create all directories");
}

struct FetchOptions {
    url: Url,
    hostname: Option<String>,
    path: Option<String>,
}
impl FetchOptions {
    fn new(
        url: &str,
    ) -> Self {
        let url = Url::parse(url).ok().unwrap();
        let hostname = Some(url.host_str().unwrap().to_string());
        let path = Some(url.path().to_string());
        FetchOptions { url, hostname, path }
    }
}

impl Default for FetchOptions {
    fn default() -> Self {
        FetchOptions::new("")
    }
}

#[async_recursion]
pub async fn fetch(url: &str, dest: &str, proxy: &str) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;

}