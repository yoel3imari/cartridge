use cartridge::integrator::desktop_file::DesktopEntryMutator;
use cartridge::integrator::icon::IconInstaller;
use cartridge::util::desktop::{DesktopEnvironment, is_in_path, refresh_desktop_databases};
use cartridge::util::fuse::{DistroInfo, is_fuse_available};
use cartridge::util::xdg::XdgPaths;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn test_distro_detection_and_packages() {
    let distro = DistroInfo::detect();
    assert!(!distro.id.is_empty());

    let pkg = distro.recommended_fuse_package();
    assert!(!pkg.is_empty());

    let cmd = distro.package_manager_install_command(&["squashfs-tools", "bubblewrap"]);
    assert!(!cmd.is_empty());

    // is_fuse_available should run safely on any host
    let _ = is_fuse_available();
}

#[test]
fn test_desktop_environment_detection() {
    let de = DesktopEnvironment::detect();
    // Should return a valid DesktopEnvironment variant without panicking
    match de {
        DesktopEnvironment::Gnome
        | DesktopEnvironment::Kde
        | DesktopEnvironment::Xfce
        | DesktopEnvironment::Cinnamon
        | DesktopEnvironment::Mate
        | DesktopEnvironment::Lxqt
        | DesktopEnvironment::Cosmic
        | DesktopEnvironment::Other(_)
        | DesktopEnvironment::Unknown => {}
    }
}

#[test]
fn test_is_in_path() {
    // /usr/bin is always in PATH on standard Linux
    assert!(is_in_path(Path::new("/usr/bin")));
    // A non-existent random directory should not be in PATH
    assert!(!is_in_path(Path::new("/nonexistent_dir_12345")));
}

#[test]
fn test_multi_resolution_icon_installation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let xdg = XdgPaths {
        home_dir: temp_dir.path().to_path_buf(),
    };
    xdg.ensure_dirs().unwrap();

    // 1x1 dummy PNG
    let dummy_png = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
        0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, // ...
        0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54,
    ];

    let installed =
        IconInstaller::install_icon(&xdg, "testapp", Some(&dummy_png), None, Some("png")).unwrap();

    assert!(!installed.is_empty());
    // Verify primary 256x256 was installed
    let primary = xdg.icons_dir().join("256x256/apps/cart-testapp.png");
    assert!(primary.exists());

    // Verify multi-resolution directories were populated for DE compatibility
    assert!(
        xdg.icons_dir()
            .join("512x512/apps/cart-testapp.png")
            .exists()
    );
    assert!(
        xdg.icons_dir()
            .join("128x128/apps/cart-testapp.png")
            .exists()
    );
    assert!(xdg.icons_dir().join("48x48/apps/cart-testapp.png").exists());
    assert!(xdg.pixmaps_dir().join("cart-testapp.png").exists());
}

#[test]
fn test_desktop_file_validator() {
    // If desktop-file-validate is available, test our generated desktop files against it
    let validate_exists = Command::new("desktop-file-validate")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !validate_exists {
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    let dummy_bin = temp_dir.path().join("app.AppImage");
    fs::write(&dummy_bin, b"#!/bin/sh\necho test\n").unwrap();

    let entry = DesktopEntryMutator::build_desktop_entry(
        None,
        "testapp",
        "Test Application",
        &dummy_bin,
        "cart-testapp",
        &["Utility".to_string()],
        Some("A test application for FreeDesktop validation"),
    );

    let desktop_path = temp_dir.path().join("testapp.desktop");
    fs::write(&desktop_path, &entry).unwrap();

    let status = Command::new("desktop-file-validate")
        .arg(&desktop_path)
        .status()
        .unwrap();

    assert!(
        status.success(),
        "Generated .desktop file failed FreeDesktop validation!"
    );
}

#[test]
fn test_refresh_desktop_databases_runs_safely() {
    let temp_dir = tempfile::tempdir().unwrap();
    let apps = temp_dir.path().join("applications");
    let icons = temp_dir.path().join("icons");
    let mime = temp_dir.path().join("mime");
    fs::create_dir_all(&apps).unwrap();
    fs::create_dir_all(&icons).unwrap();
    fs::create_dir_all(&mime).unwrap();

    // Must not panic even if running against custom or empty test directories
    refresh_desktop_databases(&apps, &icons, &mime);
}
