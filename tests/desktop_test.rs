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
    assert!(mutated.contains("StartupWMClass=existing-app"));
    assert!(mutated.contains("StartupNotify=true"));
}

#[test]
fn test_mutate_desktop_entry_with_actions() {
    let raw = r#"[Desktop Entry]
Type=Application
Name=Text Editor
Exec=editor %F
Icon=editor
Categories=TextEditor;Development;
Actions=NewWindow;NewTab;

[Desktop Action NewWindow]
Name=New Window
Exec=editor --new-window

[Desktop Action NewTab]
Name=New Tab
Exec=editor --new-tab
"#;

    let mutated = DesktopEntryMutator::build_desktop_entry(
        Some(raw),
        "text-editor",
        "Text Editor",
        Path::new("/home/user/.local/share/cartridge/apps/text-editor/editor.AppImage"),
        "cart-text-editor",
        &[],
        None,
    );

    assert!(mutated.contains(
        "Exec=\"/home/user/.local/share/cartridge/apps/text-editor/editor.AppImage\" %F"
    ));
    assert!(mutated.contains("StartupWMClass=text-editor"));
    assert!(mutated.contains(
        "Exec=\"/home/user/.local/share/cartridge/apps/text-editor/editor.AppImage\" --new-window"
    ));
    assert!(mutated.contains(
        "Exec=\"/home/user/.local/share/cartridge/apps/text-editor/editor.AppImage\" --new-tab"
    ));
}

#[test]
fn test_mutate_desktop_entry_preserves_custom_flags() {
    let raw = r#"[Desktop Entry]
Type=Application
Name=Chromium App
Exec=chromium-browser --enable-features=UseOzonePlatform %U
Icon=chromium
StartupWMClass=custom-class
Categories=Network;WebBrowser;
"#;

    let mutated = DesktopEntryMutator::build_desktop_entry(
        Some(raw),
        "chromium-app",
        "Chromium App",
        Path::new("/home/user/.local/share/cartridge/apps/chromium-app/app.AppImage"),
        "cart-chromium-app",
        &[],
        None,
    );

    assert!(mutated.contains("Exec=\"/home/user/.local/share/cartridge/apps/chromium-app/app.AppImage\" --enable-features=UseOzonePlatform %U"));
    // Preserves existing StartupWMClass
    assert!(mutated.contains("StartupWMClass=custom-class"));
}
