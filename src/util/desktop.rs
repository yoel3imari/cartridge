use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Cinnamon,
    Mate,
    Lxqt,
    Cosmic,
    Other(String),
    Unknown,
}

impl DesktopEnvironment {
    pub fn detect() -> Self {
        let xdg_current = std::env::var("XDG_CURRENT_DESKTOP")
            .unwrap_or_default()
            .to_uppercase();
        let desktop_session = std::env::var("DESKTOP_SESSION")
            .unwrap_or_default()
            .to_uppercase();

        if xdg_current.contains("GNOME") || desktop_session.contains("GNOME") {
            Self::Gnome
        } else if xdg_current.contains("KDE")
            || desktop_session.contains("KDE")
            || desktop_session.contains("PLASMA")
        {
            Self::Kde
        } else if xdg_current.contains("XFCE") || desktop_session.contains("XFCE") {
            Self::Xfce
        } else if xdg_current.contains("CINNAMON") || desktop_session.contains("CINNAMON") {
            Self::Cinnamon
        } else if xdg_current.contains("MATE") || desktop_session.contains("MATE") {
            Self::Mate
        } else if xdg_current.contains("LXQT") || desktop_session.contains("LXQT") {
            Self::Lxqt
        } else if xdg_current.contains("COSMIC") || desktop_session.contains("COSMIC") {
            Self::Cosmic
        } else if !xdg_current.is_empty() {
            Self::Other(xdg_current)
        } else {
            Self::Unknown
        }
    }
}

/// Check if a given directory is in the current process $PATH
pub fn is_in_path(target_dir: &Path) -> bool {
    let canonical_target = target_dir
        .canonicalize()
        .unwrap_or_else(|_| target_dir.to_path_buf());
    if let Some(path_var) = std::env::var_os("PATH") {
        for entry in std::env::split_paths(&path_var) {
            let canonical_entry = entry.canonicalize().unwrap_or(entry);
            if canonical_entry == canonical_target {
                return true;
            }
        }
    }
    false
}

/// Refresh all system desktop databases across GNOME, KDE Plasma, XFCE, and other desktop environments
pub fn refresh_desktop_databases(applications_dir: &Path, icons_dir: &Path, mime_dir: &Path) {
    // 1. Update FreeDesktop desktop applications database
    let _ = Command::new("update-desktop-database")
        .arg(applications_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // 2. Update FreeDesktop MIME database if directory exists
    if mime_dir.exists() {
        let _ = Command::new("update-mime-database")
            .arg(mime_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }

    // 3. Update GTK / GNOME icon caches
    let _ = Command::new("gtk-update-icon-cache")
        .arg("-f")
        .arg("-t")
        .arg(icons_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Fallback: try gtk4-update-icon-cache if available (Fedora, Arch minimal)
    let _ = Command::new("gtk4-update-icon-cache")
        .arg("-f")
        .arg("-t")
        .arg(icons_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // 4. Update KDE Plasma KSycoca (System Configuration Cache)
    // Plasma 6 (current)
    let _ = Command::new("kbuildsycoca6")
        .arg("--noincremental")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    // Plasma 5 (LTS distros: Ubuntu 22.04/24.04, Debian 12)
    let _ = Command::new("kbuildsycoca5")
        .arg("--noincremental")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}
