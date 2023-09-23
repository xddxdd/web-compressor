mod compress;

use std::env;
use std::fs;
use walkdir::WalkDir;

fn main() {
    let dir = if env::args().count() >= 2 {
        fs::canonicalize(env::args().nth(1).unwrap())
    } else {
        env::current_dir()
    };
    let dir = dir.unwrap().to_str().unwrap().to_owned();

    iterate_directory(dir);
}

fn is_compressed(file: String) -> bool {
    return file.ends_with(".gz") || file.ends_with(".br") || file.ends_with(".zstd");
}

fn iterate_directory(dir: String) {
    println!("Scanning {:?}", dir);
    let walker = WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| !is_compressed(e.path().to_str().unwrap().to_owned()));
    for it in walker {
        let entry = it.unwrap();
        let path = entry.path().to_str().unwrap().to_owned();
        // println!("Iterate {:?}", entry.path());
        if entry.file_type().is_file() {
            compress::gzip(path.clone()).unwrap();
            compress::brotli(path.clone()).unwrap();
            compress::zstd(path.clone()).unwrap();
        }
    }
}
