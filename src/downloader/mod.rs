pub mod progress;
pub mod verifier;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;

use crate::downloader::progress::create_download_progress_bar;
use crate::downloader::verifier::compute_sha256;
use crate::error::{AimError, Result};

pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("cartridge/0.1.0")
            .timeout(Duration::from_secs(600)) // 10 minutes timeout for large binaries
            .build()
            .unwrap_or_default();
        Self { client }
    }

    /// Downloads a URL to destination path with interactive progress bar
    pub async fn download_file(
        &self,
        url: &str,
        dest_path: &Path,
        label: &str,
    ) -> Result<(u64, String)> {
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let temp_dest = PathBuf::from(format!("{}.part", dest_path.display()));

        let resp = self
            .client
            .get(url)
            .send()
            .await
            .map_err(AimError::Network)?;

        if !resp.status().is_success() {
            return Err(AimError::Install(format!(
                "Download failed with HTTP status: {}",
                resp.status()
            )));
        }

        let total_size = resp.content_length();
        let pb = create_download_progress_bar(total_size, label);

        let mut file = File::create(&temp_dest)?;
        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(AimError::Network)?;
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("Done");
        drop(file);

        // Compute sha256 before finalizing
        let sha256 = compute_sha256(&temp_dest)?;

        // Rename .part to dest_path
        std::fs::rename(&temp_dest, dest_path)?;

        Ok((downloaded, sha256))
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}
