use cartridge::util::arch;

#[test]
fn test_matches_arch_x86_64() {
    assert!(arch::matches_arch("Neovim-x86_64.AppImage", "x86_64"));
    assert!(arch::matches_arch(
        "blender-v4.2-linux-x64.AppImage",
        "x86_64"
    ));
    assert!(arch::matches_arch("app-amd64.AppImage", "x86_64"));
    assert!(!arch::matches_arch("app-aarch64.AppImage", "x86_64"));
    assert!(!arch::matches_arch("app-arm64.AppImage", "x86_64"));
    assert!(!arch::matches_arch("app-i686.AppImage", "x86_64"));
    assert!(!arch::matches_arch("app-x86_64.AppImage.zsync", "x86_64"));
    assert!(!arch::matches_arch("app-x86_64.AppImage.sha256", "x86_64"));
}

#[test]
fn test_matches_arch_aarch64() {
    assert!(arch::matches_arch("app-aarch64.AppImage", "aarch64"));
    assert!(arch::matches_arch("app-arm64.AppImage", "aarch64"));
    assert!(!arch::matches_arch("app-x86_64.AppImage", "aarch64"));
    assert!(!arch::matches_arch("app-amd64.AppImage", "aarch64"));
}

#[test]
fn test_asset_score() {
    let score_exact = arch::asset_score("my-app-x86_64.AppImage", "x86_64");
    let score_generic = arch::asset_score("my-app.AppImage", "x86_64");
    assert!(score_exact > score_generic);
}
