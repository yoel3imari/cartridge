use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use walkdir::WalkDir;

use crate::error::{AimError, Result};

pub const SQUASHFS_MAGIC: [u8; 4] = [0x68, 0x73, 0x71, 0x73]; // "hsqs" in little endian

#[derive(Debug, Clone)]
pub struct AppImageInfo {
    pub is_elf: bool,
    pub is_appimage: bool,
    pub appimage_type: u8,
    pub squashfs_offset: Option<u64>,
    pub file_size: u64,
}

#[derive(Debug, Clone)]
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
            return Err(AimError::Inspection(
                "File too small to be an ELF binary".to_string(),
            ));
        }

        let is_elf = &header[0..4] == b"\x7fELF";
        let is_appimage = &header[8..10] == b"AI";
        let appimage_type = if is_appimage { header[10] } else { 0 };

        let squashfs_offset = Self::find_squashfs_offset(&mut file, file_size)?;

        Ok(AppImageInfo {
            is_elf,
            is_appimage,
            appimage_type,
            squashfs_offset,
            file_size,
        })
    }

    /// Search for the SquashFS magic bytes in the file
    fn find_squashfs_offset(file: &mut File, file_size: u64) -> Result<Option<u64>> {
        // AppImage Type 2 squashfs typically starts around 100KB to 250KB offset
        file.seek(SeekFrom::Start(0))?;
        let mut buffer = [0u8; 64 * 1024];
        let mut total_offset: u64 = 0;
        let search_limit = std::cmp::min(file_size, 5 * 1024 * 1024); // search first 5MB

        while total_offset < search_limit {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read < 4 {
                break;
            }

            for i in 0..bytes_read - 3 {
                if buffer[i..i + 4] == SQUASHFS_MAGIC {
                    return Ok(Some(total_offset + i as u64));
                }
            }

            total_offset += (bytes_read - 3) as u64;
            file.seek(SeekFrom::Start(total_offset))?;
        }

        Ok(None)
    }

    /// Extract desktop entry and icons from the AppImage
    pub fn extract_metadata(appimage_path: &Path) -> Result<ExtractedMetadata> {
        let temp_dir = tempfile::tempdir()?;
        let extract_dest = temp_dir.path().join("squashfs-root");

        let info = Self::inspect_file(appimage_path).unwrap_or(AppImageInfo {
            is_elf: true,
            is_appimage: true,
            appimage_type: 2,
            squashfs_offset: None,
            file_size: 0,
        });

        let mut extraction_succeeded = false;

        // Strategy 1: Use unsquashfs if installed and offset was found
        if let Some(offset) = info.squashfs_offset {
            let status = Command::new("unsquashfs")
                .arg("-offset")
                .arg(offset.to_string())
                .arg("-dest")
                .arg(&extract_dest)
                .arg(appimage_path)
                .arg("*.desktop")
                .arg(".DirIcon")
                .arg("*.png")
                .arg("*.svg")
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

        // Strategy 2: If unsquashfs failed or not available, use AppImage's own --appimage-extract
        if !extraction_succeeded {
            // Ensure executable permission
            if let Ok(meta) = fs::metadata(appimage_path) {
                let mut perms = meta.permissions();
                perms.set_mode(perms.mode() | 0o755);
                let _ = fs::set_permissions(appimage_path, perms);
            }

            let status = Command::new(appimage_path)
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
            return Ok(ExtractedMetadata {
                desktop_content: None,
                desktop_file_name: None,
                icon_path: None,
                icon_bytes: None,
                icon_extension: None,
            });
        }

        // Parse extracted files
        let mut desktop_content = None;
        let mut desktop_file_name = None;
        let mut icon_path = None;
        let mut icon_bytes = None;
        let mut icon_extension = None;

        for entry in WalkDir::new(&extract_dest)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Look for .desktop file
            if file_name.ends_with(".desktop")
                && desktop_content.is_none()
                && let Ok(content) = fs::read_to_string(path)
            {
                desktop_content = Some(content);
                desktop_file_name = Some(file_name.to_string());
            }

            // Look for .DirIcon or best icon
            if file_name == ".DirIcon" {
                if let Ok(bytes) = fs::read(path) {
                    // Check if SVG or PNG
                    let ext = if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
                        "svg"
                    } else {
                        "png"
                    };
                    icon_path = Some(path.to_path_buf());
                    icon_bytes = Some(bytes);
                    icon_extension = Some(ext.to_string());
                }
            } else if icon_bytes.is_none()
                && (file_name.ends_with(".png") || file_name.ends_with(".svg"))
            {
                let ext = if file_name.ends_with(".svg") {
                    "svg"
                } else {
                    "png"
                };
                if let Ok(bytes) = fs::read(path) {
                    icon_path = Some(path.to_path_buf());
                    icon_bytes = Some(bytes);
                    icon_extension = Some(ext.to_string());
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
