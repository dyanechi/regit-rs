use std::{env::{Args, self}, path::Path};

use degit_rs::{util::path_exists, Degit};

#[tokio::main]
async fn main() {
    let mut args = env::args();
    args.next();
    if args.len() < 2 {
        panic!("INVALID_ARGS: 2 args are required");
    }
    let (src, dest) = (args.next().unwrap(), &args.next().unwrap());
    let src = Path::new(&src);
    let dest = Path::new(&dest);

    let empty = !path_exists(dest);
    if !empty {

    }
    
    run(src, dest).await;
}


async fn run(src: &Path, dest: &Path) {
    let d = Degit::new(src.to_str().unwrap())
        .build();

    d.clone(dest).await;
}
