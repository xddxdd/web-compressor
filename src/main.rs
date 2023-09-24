mod compress;

use std::env;
use std::fs;
use walkdir::WalkDir;

fn main() {
    if env::args().count() != 2 {
        let argv_0 = env::args().nth(0).unwrap();
        println!("Usage: {} path/to/compress", argv_0);
        std::process::exit(1);
    }

    let dir = fs::canonicalize(env::args().nth(1).unwrap())
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    iterate_directory(dir);
}

fn is_compressed(file: String) -> bool {
    return file.ends_with(".gz") || file.ends_with(".br") || file.ends_with(".zstd");
}

fn iterate_directory(dir: String) {
    rayon::scope(|scope| {
        println!("Scanning {:?}", dir);
        let walker = WalkDir::new(dir)
            .into_iter()
            .filter_entry(|e| !is_compressed(e.path().to_str().unwrap().to_owned()));

        for it in walker {
            let entry = it.unwrap();
            let path = entry.path().to_str().unwrap().to_owned();

            if !entry.file_type().is_file() {
                continue;
            }

            scope.spawn(move |_| {
                let max_size = match compress::file_size(path.clone()) {
                    Ok(len) => len,
                    Err(e) => {
                        println!("Error processing {}: {}", path.clone(), e.to_string());
                        return;
                    }
                };

                // Sorted by browser compatibility, prefer older format if smaller
                let size = compress::gzip(path.clone(), max_size).unwrap();
                let max_size = std::cmp::min(max_size, size);
                let size = compress::brotli(path.clone(), max_size).unwrap();
                let max_size = std::cmp::min(max_size, size);
                let size = compress::zstd(path.clone(), max_size).unwrap();
                let max_size = std::cmp::min(max_size, size);

                // Suppress unused code warning
                let _ = max_size;
            });
        }
    });
}
