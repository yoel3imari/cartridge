use aim::manager::state::{AppRegistry, InstalledApp};
use std::path::PathBuf;

#[test]
fn test_state_store_registration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let state_file = temp_dir.path().join("installed.json");

    let mut registry = AppRegistry::default();
    let app = InstalledApp {
        id: "test-app".to_string(),
        name: "Test App".to_string(),
        version: "1.0.0".to_string(),
        source: "gh:test/app".to_string(),
        binary_path: PathBuf::from("/tmp/test.AppImage"),
        backup_binary_path: None,
        desktop_file: Some(PathBuf::from("/tmp/test.desktop")),
        icon_paths: vec![PathBuf::from("/tmp/test.png")],
        symlink_path: Some(PathBuf::from("/tmp/test")),
        installed_at: "2026-10-07T12:00:00Z".to_string(),
        sha256: Some("abcdef123456".to_string()),
        file_size: 1048576,
    };

    registry.apps.insert(app.id.clone(), app);

    // Save
    let json = serde_json::to_string_pretty(&registry).unwrap();
    std::fs::write(&state_file, json).unwrap();

    // Reload
    let content = std::fs::read_to_string(&state_file).unwrap();
    let loaded: AppRegistry = serde_json::from_str(&content).unwrap();

    assert_eq!(loaded.apps.len(), 1);
    let loaded_app = loaded.apps.get("test-app").unwrap();
    assert_eq!(loaded_app.name, "Test App");
    assert_eq!(loaded_app.version, "1.0.0");
    assert_eq!(loaded_app.file_size, 1048576);
}
