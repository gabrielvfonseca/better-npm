//! Compression and decompression support (gzip, zstd, deflate).
//!
//! Provides helpers for compressing and decompressing byte buffers
//! and files using various algorithms.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::Path;

/// Error type for compression operations.
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("compression error: {0}")]
    Compression(String),
    #[error("decompression error: {0}")]
    Decompression(String),
    #[error("unsupported algorithm: {0}")]
    UnsupportedAlgorithm(String),
}

/// Supported compression algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompressionAlgo {
    /// Gzip compression.
    Gzip,
    /// Zstandard compression.
    Zstd,
    /// Deflate (raw) compression.
    Deflate,
}

impl CompressionAlgo {
    /// Return the file extension for this algorithm.
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionAlgo::Gzip => ".gz",
            CompressionAlgo::Zstd => ".zst",
            CompressionAlgo::Deflate => ".deflate",
        }
    }
}

/// Compress the given data using the specified algorithm.
pub fn compress(data: &[u8], algo: CompressionAlgo) -> Result<Vec<u8>, CompressionError> {
    match algo {
        CompressionAlgo::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(data)?;
            encoder.finish().map_err(|e| CompressionError::Compression(e.to_string()))
        }
        CompressionAlgo::Zstd => {
            let compressed = zstd::encode_all(std::io::Cursor::new(data), 3)
                .map_err(|e| CompressionError::Compression(e.to_string()))?;
            Ok(compressed)
        }
        CompressionAlgo::Deflate => {
            let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(data)?;
            encoder.finish().map_err(|e| CompressionError::Compression(e.to_string()))
        }
    }
}

/// Decompress the given data using the specified algorithm.
pub fn decompress(data: &[u8], algo: CompressionAlgo) -> Result<Vec<u8>, CompressionError> {
    match algo {
        CompressionAlgo::Gzip => {
            let mut decoder = flate2::read::GzDecoder::new(data);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
        CompressionAlgo::Zstd => {
            let decompressed = zstd::decode_all(std::io::Cursor::new(data))
                .map_err(|e| CompressionError::Decompression(e.to_string()))?;
            Ok(decompressed)
        }
        CompressionAlgo::Deflate => {
            let mut decoder = flate2::read::DeflateDecoder::new(data);
            let mut out = Vec::new();
            decoder.read_to_end(&mut out)?;
            Ok(out)
        }
    }
}

/// Compress a file in-place, replacing the original with the compressed version.
pub fn compress_file(path: &Path, algo: CompressionAlgo) -> Result<(), CompressionError> {
    let data = std::fs::read(path)?;
    let compressed = compress(&data, algo)?;
    std::fs::write(path, &compressed)?;
    Ok(())
}

/// Decompress a file in-place, replacing the original with the decompressed version.
pub fn decompress_file(path: &Path, algo: CompressionAlgo) -> Result<(), CompressionError> {
    let data = std::fs::read(path)?;
    let decompressed = decompress(&data, algo)?;
    std::fs::write(path, &decompressed)?;
    Ok(())
}

/// Compress data asynchronously using tokio task.
pub async fn compress_async(data: Vec<u8>, algo: CompressionAlgo) -> Result<Vec<u8>, CompressionError> {
    tokio::task::spawn_blocking(move || compress(&data, algo))
        .await
        .map_err(|e| CompressionError::Compression(e.to_string()))?
}

/// Decompress data asynchronously using tokio task.
pub async fn decompress_async(data: Vec<u8>, algo: CompressionAlgo) -> Result<Vec<u8>, CompressionError> {
    tokio::task::spawn_blocking(move || decompress(&data, algo))
        .await
        .map_err(|e| CompressionError::Decompression(e.to_string()))?
}

/// Stream a reader through a compressor, writing to a writer.
pub fn compress_stream<R: Read, W: Write>(
    mut reader: R,
    writer: W,
    algo: CompressionAlgo,
) -> Result<u64, CompressionError> {
    match algo {
        CompressionAlgo::Gzip => {
            let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
            let bytes = std::io::copy(&mut reader, &mut encoder)?;
            encoder.finish()?;
            Ok(bytes)
        }
        CompressionAlgo::Zstd => {
            let mut encoder = zstd::Encoder::new(writer, 3)
                .map_err(|e| CompressionError::Compression(e.to_string()))?;
            let bytes = std::io::copy(&mut reader, &mut encoder)?;
            encoder.finish()
                .map_err(|e| CompressionError::Compression(e.to_string()))?;
            Ok(bytes)
        }
        CompressionAlgo::Deflate => {
            let mut encoder = flate2::write::DeflateEncoder::new(writer, flate2::Compression::default());
            let bytes = std::io::copy(&mut reader, &mut encoder)?;
            encoder.finish()?;
            Ok(bytes)
        }
    }
}

/// Stream a reader through a decompressor, writing to a writer.
pub fn decompress_stream<R: Read, W: Write>(
    reader: R,
    writer: W,
    algo: CompressionAlgo,
) -> Result<u64, CompressionError> {
    match algo {
        CompressionAlgo::Gzip => {
            let mut decoder = flate2::read::GzDecoder::new(reader);
            let mut writer = writer;
            Ok(std::io::copy(&mut decoder, &mut writer)?)
        }
        CompressionAlgo::Zstd => {
            let mut decoder = zstd::Decoder::new(reader)
                .map_err(|e| CompressionError::Decompression(e.to_string()))?;
            let mut writer = writer;
            Ok(std::io::copy(&mut decoder, &mut writer)?)
        }
        CompressionAlgo::Deflate => {
            let mut decoder = flate2::read::DeflateDecoder::new(reader);
            let mut writer = writer;
            Ok(std::io::copy(&mut decoder, &mut writer)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_gzip() {
        let data = b"hello world, this is test data for gzip compression!";
        let compressed = compress(data, CompressionAlgo::Gzip).unwrap();
        assert_ne!(compressed, data);
        let decompressed = decompress(&compressed, CompressionAlgo::Gzip).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_zstd() {
        let data = b"hello world, zstd compression test!";
        let compressed = compress(data, CompressionAlgo::Zstd).unwrap();
        assert_ne!(compressed, data);
        let decompressed = decompress(&compressed, CompressionAlgo::Zstd).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_decompress_deflate() {
        let data = b"hello world, deflate compression test!";
        let compressed = compress(data, CompressionAlgo::Deflate).unwrap();
        assert_ne!(compressed, data);
        let decompressed = decompress(&compressed, CompressionAlgo::Deflate).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compress_empty() {
        let data = b"";
        for algo in &[CompressionAlgo::Gzip, CompressionAlgo::Zstd, CompressionAlgo::Deflate] {
            let compressed = compress(data, *algo).unwrap();
            let decompressed = decompress(&compressed, *algo).unwrap();
            assert_eq!(decompressed, data);
        }
    }

    #[test]
    fn test_compress_stream() {
        let data = b"stream test data";
        let mut output = Vec::new();
        compress_stream(&data[..], &mut output, CompressionAlgo::Gzip).unwrap();
        let mut decompressed = Vec::new();
        decompress_stream(&output[..], &mut decompressed, CompressionAlgo::Gzip).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_algorithm_extension() {
        assert_eq!(CompressionAlgo::Gzip.extension(), ".gz");
        assert_eq!(CompressionAlgo::Zstd.extension(), ".zst");
        assert_eq!(CompressionAlgo::Deflate.extension(), ".deflate");
    }
}
