use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

const CHUNK_SIZE: usize = 64 * 1024;

fn chunked_read_write(file: String, writer: &mut Box<dyn Write>) -> Result<u64, std::io::Error> {
    let mut input_file = match File::open(file.clone()) {
        Ok(file) => file,
        Err(error) => return Err(error),
    };

    let input_size = match Read::by_ref(&mut input_file).metadata() {
        Ok(metadata) => metadata.len(),
        Err(error) => return Err(error),
    };

    let mut reader = BufReader::with_capacity(CHUNK_SIZE, input_file);

    loop {
        let buffer = match reader.fill_buf() {
            Ok(buf) => buf.to_owned(),
            Err(error) => return Err(error),
        };
        reader.consume(buffer.len());
        if buffer.len() > 0 {
            match writer.write_all(&buffer) {
                Ok(_) => (),
                Err(error) => return Err(error),
            };
        }
        if buffer.len() < CHUNK_SIZE {
            break;
        }
    }

    match writer.flush() {
        Ok(_) => (),
        Err(error) => return Err(error),
    };

    return Ok(input_size);
}

fn compress_to_writer(
    file: String,
    compress_name: &str,
    compress_ext: &str,
    writer: fn(w: Box<dyn Write>) -> Box<dyn Write>,
) -> Result<(), std::io::Error> {
    let output_path = format!("{}.{}", file, compress_ext);

    if Path::new(&output_path).exists() {
        println!("File already {} compressed: {}", compress_name, file);
        return Ok(());
    }

    let mut output_file = match File::create(output_path.clone()) {
        Ok(f) => f,
        Err(error) => return Err(error),
    };

    let mut e = writer(match output_file.try_clone() {
        Ok(f) => Box::new(f),
        Err(error) => return Err(error),
    });

    let input_size = match chunked_read_write(file.clone(), &mut e) {
        Ok(size) => size,
        Err(error) => return Err(error),
    };

    let output_size = match Read::by_ref(&mut output_file).metadata() {
        Ok(metadata) => metadata.len(),
        Err(error) => return Err(error),
    };

    std::mem::drop(output_file);

    if output_size < input_size {
        println!(
            "{} compress {}: {} -> {} bytes",
            compress_name, file, input_size, output_size
        );
    } else {
        println!(
            "{} compress {}: {} -> {} bytes, deleting output",
            compress_name, file, input_size, output_size
        );

        match fs::remove_file(output_path.clone()) {
            Ok(_) => (),
            Err(error) => return Err(error),
        }
    }
    return Ok(());
}

pub fn gzip(file: String) -> Result<(), std::io::Error> {
    compress_to_writer(file, "Gzip", "gz", |w| {
        Box::new(GzEncoder::new(w, Compression::best()))
    })
}

pub fn brotli(file: String) -> Result<(), std::io::Error> {
    compress_to_writer(file, "Brotli", "br", |w| {
        Box::new(brotli::CompressorWriter::new(w, CHUNK_SIZE, 11, 22))
    })
}

pub fn zstd(file: String) -> Result<(), std::io::Error> {
    compress_to_writer(file, "Zstd", "zst", |w| {
        Box::new(zstd::Encoder::new(w, 19).unwrap())
    })
}
