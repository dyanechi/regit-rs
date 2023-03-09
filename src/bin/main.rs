#![macro_use]
use std::{env::{Args, self}, path::Path};

use regit::{app::{Regit, RegitOptions}};

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
    let mut regit = Regit::new(src, RegitOptions::default());
    regit.clone(dest).await;
}
