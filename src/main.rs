use flate2::write::GzEncoder;
use flate2::Compression;
use std::env;
use std::fs;
use std::fs::File;
use std::fs::FileType;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
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
            compress_gz(path);
        }
    }
}

fn compress_gz(file: String) {
    let output_path = format!("{}.gz", file);

    if Path::new(&output_path).exists() {
        println!("File already gzipped: {}", file.clone());
        return;
    }

    let mut input_file = match File::open(file.clone()) {
        Ok(file) => file,
        Err(error) => {
            println!("Cannot open {}: {}", file.clone(), error);
            return;
        }
    };
    let input_size = Read::by_ref(&mut input_file).metadata().unwrap().len();

    let mut output_file = File::create(output_path.clone()).unwrap();
    let mut e = GzEncoder::new(output_file.try_clone().unwrap(), Compression::best());

    let chunk_size = 64 * 1024;
    let mut reader = BufReader::with_capacity(chunk_size, input_file);
    loop {
        let buffer = reader.fill_buf().unwrap().to_owned();
        reader.consume(buffer.len());
        if buffer.len() > 0 {
            e.write_all(&buffer).unwrap();
        }
        if buffer.len() < chunk_size {
            break;
        }
    }
    e.finish().unwrap();

    let output_size = Read::by_ref(&mut output_file).metadata().unwrap().len();

    std::mem::drop(output_file);

    if output_size < input_size {
        println!("Gzip {}: {} -> {} bytes", file, input_size, output_size);
    } else {
        println!(
            "Gzip {}: {} -> {} bytes, deleting output",
            file, input_size, output_size
        );
        fs::remove_file(output_path.clone()).unwrap()
    }
}
