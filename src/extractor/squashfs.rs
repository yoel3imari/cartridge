use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

use crate::error::{CartridgeError, Result};

pub const SQUASHFS_MAGIC: [u8; 4] = [0x68, 0x73, 0x71, 0x73]; // "hsqs" in little endian

#[derive(Debug, Clone)]
pub struct AppImageInfo {
    pub is_elf: bool,
    pub is_appimage: bool,
    pub appimage_type: u8,
    pub squashfs_offset: Option<u64>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedMetadata {
    pub desktop_content: Option<String>,
    pub desktop_file_name: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub icon_bytes: Option<Vec<u8>>,
    pub icon_extension: Option<String>,
}

pub struct AppImageExtractor;

impl AppImageExtractor {
    /// Inspect an AppImage file to verify headers and detect SquashFS offset
    pub fn inspect_file(path: &Path) -> Result<AppImageInfo> {
        let mut file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        let mut header = [0u8; 64];
        let bytes_read = file.read(&mut header)?;
        if bytes_read < 16 {
            return Err(CartridgeError::Inspection(
                "File too small to be an ELF binary".to_string(),
            ));
        }

        let is_elf = &header[0..4] == b"\x7fELF";
        let is_appimage = &header[8..10] == b"AI";
        let appimage_type = if is_appimage { header[10] } else { 0 };

        let offsets = Self::find_squashfs_offsets(&mut file, file_size)?;
        let squashfs_offset = offsets.first().copied();

        Ok(AppImageInfo {
            is_elf,
            is_appimage,
            appimage_type,
            squashfs_offset,
            file_size,
        })
    }

    /// Check if a 32-byte header represents a valid SquashFS 4 superblock
    pub fn is_valid_squashfs_superblock(data: &[u8]) -> bool {
        if data.len() < 32 {
            return false;
        }
        // magic is "hsqs"
        if data[0..4] != SQUASHFS_MAGIC {
            return false;
        }
        // block size: offset 12..16 (u32 little-endian)
        let block_size = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        if !(4096..=1_048_576).contains(&block_size) || (block_size & (block_size - 1)) != 0 {
            return false;
        }
        // compression: offset 20..22 (u16 little-endian: 1=gzip, 2=lzma, 3=lzo, 4=xz, 5=lz4, 6=zstd)
        let compression = u16::from_le_bytes([data[20], data[21]]);
        if !(1..=6).contains(&compression) {
            return false;
        }
        // s_major: offset 28..30, s_minor: offset 30..32
        let s_major = u16::from_le_bytes([data[28], data[29]]);
        let s_minor = u16::from_le_bytes([data[30], data[31]]);

        s_major == 4 && s_minor == 0
    }

    /// Search for candidate SquashFS offsets in the file, prioritizing verified superblocks
    pub fn find_squashfs_offsets(file: &mut File, file_size: u64) -> Result<Vec<u64>> {
        file.seek(SeekFrom::Start(0))?;
        let mut buffer = [0u8; 64 * 1024];
        let mut total_offset: u64 = 0;
        let search_limit = std::cmp::min(file_size, 50 * 1024 * 1024); // search first 50MB

        let mut verified_offsets = Vec::new();
        let mut fallback_offsets = Vec::new();

        while total_offset < search_limit {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read < 4 {
                break;
            }

            for i in 0..bytes_read - 3 {
                if buffer[i..i + 4] == SQUASHFS_MAGIC {
                    let offset = total_offset + i as u64;
                    if i + 32 <= bytes_read {
                        if Self::is_valid_squashfs_superblock(&buffer[i..i + 32]) {
                            verified_offsets.push(offset);
                        } else {
                            fallback_offsets.push(offset);
                        }
                    } else {
                        let current_pos = file.stream_position().unwrap_or(0);
                        let mut check_buf = [0u8; 32];
                        if file.seek(SeekFrom::Start(offset)).is_ok()
                            && file.read_exact(&mut check_buf).is_ok()
                            && Self::is_valid_squashfs_superblock(&check_buf)
                        {
                            verified_offsets.push(offset);
                        } else {
                            fallback_offsets.push(offset);
                        }
                        let _ = file.seek(SeekFrom::Start(current_pos));
                    }
                }
            }

            total_offset += (bytes_read - 3) as u64;
            file.seek(SeekFrom::Start(total_offset))?;
        }

        if !verified_offsets.is_empty() {
            Ok(verified_offsets)
        } else {
            Ok(fallback_offsets)
        }
    }

    /// Extract desktop entry and icons from the AppImage
    pub fn extract_metadata(appimage_path: &Path) -> Result<ExtractedMetadata> {
        let canonical_path = appimage_path
            .canonicalize()
            .unwrap_or_else(|_| appimage_path.to_path_buf());

        let temp_dir = tempfile::tempdir()?;
        let extract_dest = temp_dir.path().join("squashfs-root");

        let mut offsets = Vec::new();
        if let Ok(mut file) = File::open(&canonical_path)
            && let Ok(metadata) = file.metadata()
        {
            offsets = Self::find_squashfs_offsets(&mut file, metadata.len()).unwrap_or_default();
        }

        let mut extraction_succeeded = false;

        // Strategy 1: Use unsquashfs if installed with each discovered offset
        for offset in &offsets {
            let status = Command::new("unsquashfs")
                .arg("-offset")
                .arg(offset.to_string())
                .arg("-dest")
                .arg(&extract_dest)
                .arg(&canonical_path)
                .arg("*.desktop")
                .arg(".DirIcon")
                .arg("*.png")
                .arg("*.svg")
                .arg("usr/share/icons/*")
                .arg("usr/share/applications/*")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if let Ok(st) = status
                && st.success()
                && extract_dest.exists()
            {
                extraction_succeeded = true;
                break;
            }
        }

        // Strategy 2: If unsquashfs failed or not available, use AppImage's own --appimage-extract
        if !extraction_succeeded {
            // Ensure executable permission
            if let Ok(meta) = fs::metadata(&canonical_path) {
                let mut perms = meta.permissions();
                perms.set_mode(perms.mode() | 0o755);
                let _ = fs::set_permissions(&canonical_path, perms);
            }

            let status = Command::new(&canonical_path)
                .arg("--appimage-extract")
                .current_dir(temp_dir.path())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();

            if let Ok(st) = status
                && st.success()
                && extract_dest.exists()
            {
                extraction_succeeded = true;
            }
        }

        if !extraction_succeeded && !extract_dest.exists() {
            return Ok(ExtractedMetadata::default());
        }

        // Parse extracted files
        let mut desktop_content = None;
        let mut desktop_file_name = None;
        let mut icon_path = None;
        let mut icon_bytes = None;
        let mut icon_extension = None;
        let mut max_icon_size = 0;

        for entry in WalkDir::new(&extract_dest)
            .max_depth(8)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Look for .desktop file
            if file_name.ends_with(".desktop")
                && !file_name.starts_with('.')
                && let Ok(content) = fs::read_to_string(path)
                && (desktop_content.is_none() || content.contains("Name="))
            {
                desktop_content = Some(content);
                desktop_file_name = Some(file_name.to_string());
            }

            // Look for icons
            if file_name == ".DirIcon" {
                if let Ok(bytes) = fs::read(path) {
                    let ext = if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
                        "svg"
                    } else {
                        "png"
                    };
                    icon_path = Some(path.to_path_buf());
                    icon_bytes = Some(bytes);
                    icon_extension = Some(ext.to_string());
                    max_icon_size = usize::MAX; // .DirIcon has highest priority
                }
            } else if max_icon_size < usize::MAX
                && (file_name.ends_with(".png") || file_name.ends_with(".svg"))
                && !file_name.starts_with('.')
            {
                let ext = if file_name.ends_with(".svg") {
                    "svg"
                } else {
                    "png"
                };
                if let Ok(bytes) = fs::read(path) {
                    let score = if ext == "svg" {
                        10_000_000
                    } else {
                        bytes.len()
                    };
                    if score > max_icon_size {
                        max_icon_size = score;
                        icon_path = Some(path.to_path_buf());
                        icon_bytes = Some(bytes);
                        icon_extension = Some(ext.to_string());
                    }
                }
            }
        }

        Ok(ExtractedMetadata {
            desktop_content,
            desktop_file_name,
            icon_path,
            icon_bytes,
            icon_extension,
        })
    }
}
