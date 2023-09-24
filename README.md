# Web-Compressor

A simple Rust program that pre-compresses your static web assets.

## Usage

```bash
./web-compressor path/to/compress
```

This program will compress **everything** in the target folder into:

- Gzip `.gz`: only if smaller than original file
- Brotli `.br`: only if smaller than original file and Gzip
- Zstd `.zst`: only if smaller than original file, Gzip and Brotli

This program will run compression steps in parallel, utilizing all CPU cores.

## License

Unlicense.
