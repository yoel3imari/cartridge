use cartridge::integrator::desktop_file::DesktopEntryMutator;
use std::path::Path;

#[test]
fn test_generate_new_desktop_entry() {
    let entry = DesktopEntryMutator::build_desktop_entry(
        None,
        "kdenlive",
        "Kdenlive Video Editor",
        Path::new("/home/user/.local/share/cartridge/apps/kdenlive/kdenlive.AppImage"),
        "cart-kdenlive",
        &["AudioVideo".to_string(), "Video".to_string()],
        Some("Video Editing Software"),
    );

    assert!(entry.contains("[Desktop Entry]"));
    assert!(entry.contains("Name=Kdenlive Video Editor"));
    assert!(
        entry.contains(
            "Exec=\"/home/user/.local/share/cartridge/apps/kdenlive/kdenlive.AppImage\" %U"
        )
    );
    assert!(entry.contains("Icon=cart-kdenlive"));
    assert!(entry.contains("Categories=AudioVideo;Video;"));
    assert!(entry.contains("StartupWMClass=kdenlive"));
}

#[test]
fn test_mutate_existing_desktop_entry() {
    let existing = r#"[Desktop Entry]
Type=Application
Name=Existing App
Comment=An existing application
Exec=appimage-exec %u
Icon=app-icon
Categories=Utility;
"#;

    let mutated = DesktopEntryMutator::build_desktop_entry(
        Some(existing),
        "existing-app",
        "Existing App",
        Path::new("/home/user/.local/share/cartridge/apps/existing-app/app.AppImage"),
        "cart-existing-app",
        &[],
        None,
    );

    assert!(mutated.contains("Name=Existing App"));
    assert!(
        mutated.contains(
            "Exec=\"/home/user/.local/share/cartridge/apps/existing-app/app.AppImage\" %U"
        )
    );
    assert!(mutated.contains("Icon=cart-existing-app"));
    assert!(
        mutated
            .contains("TryExec=/home/user/.local/share/cartridge/apps/existing-app/app.AppImage")
    );
    assert!(mutated.contains("Comment=An existing application"));
}
