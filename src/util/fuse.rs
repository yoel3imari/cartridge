use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistroInfo {
    pub id: String,
    pub id_like: Vec<String>,
    pub version_id: String,
    pub pretty_name: String,
}

impl DistroInfo {
    pub fn detect() -> Self {
        let os_release_paths = ["/etc/os-release", "/usr/lib/os-release"];
        for path in &os_release_paths {
            if let Ok(content) = fs::read_to_string(path) {
                return Self::parse(&content);
            }
        }
        Self {
            id: "linux".to_string(),
            id_like: Vec::new(),
            version_id: String::new(),
            pretty_name: "Linux".to_string(),
        }
    }

    pub fn parse(content: &str) -> Self {
        let mut id = "linux".to_string();
        let mut id_like = Vec::new();
        let mut version_id = String::new();
        let mut pretty_name = "Linux".to_string();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || !line.contains('=') {
                continue;
            }
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            match key {
                "ID" => id = val.to_lowercase(),
                "ID_LIKE" => id_like = val.split_whitespace().map(|s| s.to_lowercase()).collect(),
                "VERSION_ID" => version_id = val.to_string(),
                "PRETTY_NAME" => pretty_name = val.to_string(),
                _ => {}
            }
        }

        Self {
            id,
            id_like,
            version_id,
            pretty_name,
        }
    }

    pub fn is_ubuntu_or_derivative(&self) -> bool {
        self.id == "ubuntu" || self.id_like.iter().any(|s| s == "ubuntu" || s == "debian")
    }

    pub fn is_fedora_or_rhel(&self) -> bool {
        self.id == "fedora"
            || self.id == "rhel"
            || self.id == "centos"
            || self.id_like.iter().any(|s| s == "fedora" || s == "rhel")
    }

    pub fn is_arch(&self) -> bool {
        self.id == "arch" || self.id_like.iter().any(|s| s == "arch")
    }

    pub fn is_opensuse(&self) -> bool {
        self.id.contains("suse") || self.id_like.iter().any(|s| s.contains("suse"))
    }

    pub fn recommended_fuse_package(&self) -> &'static str {
        if self.id == "ubuntu" {
            // Ubuntu 24.04+ (Noble Numbat and newer) uses libfuse2t64 for 64-bit time_t
            if let Ok(ver) = self.version_id.parse::<f32>()
                && ver >= 24.0
            {
                return "libfuse2t64";
            }
            return "libfuse2";
        }
        if self.is_ubuntu_or_derivative() {
            return "libfuse2";
        }
        if self.is_fedora_or_rhel() {
            return "fuse-libs";
        }
        if self.is_arch() {
            return "fuse2";
        }
        if self.is_opensuse() {
            return "libfuse2";
        }
        if self.id == "alpine" {
            return "fuse";
        }
        "libfuse2"
    }

    pub fn package_manager_install_command(&self, packages: &[&str]) -> String {
        let pkg_list = packages.join(" ");
        if self.is_ubuntu_or_derivative() {
            format!("sudo apt install {}", pkg_list)
        } else if self.is_fedora_or_rhel() {
            format!("sudo dnf install {}", pkg_list)
        } else if self.is_arch() {
            format!("sudo pacman -S {}", pkg_list)
        } else if self.is_opensuse() {
            format!("sudo zypper install {}", pkg_list)
        } else if self.id == "alpine" {
            format!("sudo apk add {}", pkg_list)
        } else {
            format!("install {}", pkg_list)
        }
    }
}

/// Check if FUSE 2 is available on the host system to run AppImages natively
pub fn is_fuse_available() -> bool {
    // If environment explicitly forces extract mode, report false
    if std::env::var("APPIMAGE_EXTRACT_AND_RUN").as_deref() == Ok("1") {
        return false;
    }

    // Check /dev/fuse
    if !Path::new("/dev/fuse").exists() {
        return false;
    }

    // Check standard dynamic library locations for libfuse.so.2 across all distros
    let common_lib_dirs = [
        "/lib",
        "/lib64",
        "/usr/lib",
        "/usr/lib64",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/usr/lib/arm-linux-gnueabihf",
        "/usr/lib/i386-linux-gnu",
        "/usr/local/lib",
        "/usr/local/lib64",
    ];

    for dir in &common_lib_dirs {
        if Path::new(dir).join("libfuse.so.2").exists() {
            return true;
        }
    }

    // Check ldconfig cache if available
    if let Ok(output) = Command::new("ldconfig").arg("-p").output() {
        let s = String::from_utf8_lossy(&output.stdout);
        if s.contains("libfuse.so.2") {
            return true;
        }
    }

    false
}
