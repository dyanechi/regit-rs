use std::{env::{Args, self}, path::Path};

// use degit_rs::{util::path_exists, Degit};
use regit::{app::{Regit, RegitOptions}, util::path_exists};

#[tokio::main]
async fn main() {
    let mut args = env::args();
    args.next();
    if args.len() < 2 {
        panic!("INVALID_ARGS: 2 args are required");
    }
    let (src, dest) = (args.next().unwrap(), args.next().unwrap()); 
    
    run(&src, &dest).await;
}


async fn run(src: &str, dest: &str) {
    if !Path::new(dest).exists() {
        panic!("destination '{}' doesn't exist", dest);
    }
    let regit = Regit::new(src, RegitOptions::default());
    regit.clone(dest).await;
    // d.clone(dest).await;
}
