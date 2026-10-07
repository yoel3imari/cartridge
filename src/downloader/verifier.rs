use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{CartridgeError, Result};

pub fn compute_sha256(path: &Path) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn verify_checksum(path: &Path, expected_hash: &str) -> Result<()> {
    let actual_hash = compute_sha256(path)?;
    if !actual_hash.eq_ignore_ascii_case(expected_hash.trim()) {
        return Err(CartridgeError::Install(format!(
            "Checksum mismatch! Expected: {}, Computed: {}",
            expected_hash, actual_hash
        )));
    }
    Ok(())
}
