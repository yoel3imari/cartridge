use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus};

use crate::error::{CartridgeError, Result};

pub struct SandboxRunner;

impl SandboxRunner {
    /// Verify that bwrap is installed on the system
    pub fn is_available() -> bool {
        Command::new("bwrap")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Execute an AppImage inside an isolated Bubblewrap sandbox
    pub fn run_sandboxed(
        binary_path: &Path,
        args: &[String],
        allow_network: bool,
    ) -> Result<ExitStatus> {
        if !Self::is_available() {
            return Err(CartridgeError::Sandbox(
                "Bubblewrap ('bwrap') is not installed or not found in PATH.".to_string(),
            ));
        }

        let mut cmd = Command::new("bwrap");

        // 1. Base read-only system binds
        cmd.arg("--ro-bind").arg("/usr").arg("/usr");
        cmd.arg("--ro-bind-try").arg("/lib").arg("/lib");
        cmd.arg("--ro-bind-try").arg("/lib64").arg("/lib64");
        cmd.arg("--ro-bind-try").arg("/bin").arg("/bin");
        cmd.arg("--ro-bind-try").arg("/sbin").arg("/sbin");
        cmd.arg("--ro-bind-try").arg("/etc").arg("/etc");
        cmd.arg("--ro-bind-try").arg("/opt").arg("/opt");
        cmd.arg("--ro-bind-try").arg("/usr/local/share").arg("/usr/local/share");

        // 2. Dynamic system devices and runtime
        cmd.arg("--proc").arg("/proc");
        cmd.arg("--dev").arg("/dev");
        cmd.arg("--tmpfs").arg("/tmp");
        cmd.arg("--tmpfs").arg("/run");
        cmd.arg("--tmpfs").arg("/dev/shm"); // POSIX shared memory for Chromium/Electron/Qt/GTK

        // 3. DNS and network state forwarding when inside tmpfs /run
        cmd.arg("--ro-bind-try").arg("/run/systemd/resolve").arg("/run/systemd/resolve");
        cmd.arg("--ro-bind-try").arg("/run/NetworkManager").arg("/run/NetworkManager");
        cmd.arg("--ro-bind-try").arg("/run/resolvconf").arg("/run/resolvconf");
        cmd.arg("--ro-bind-try").arg("/run/connman").arg("/run/connman");
        cmd.arg("--ro-bind-try").arg("/run/netns").arg("/run/netns");

        // If /etc/resolv.conf is a symlink pointing into /run, bind its canonical destination
        if let Ok(target) = fs::canonicalize("/etc/resolv.conf")
            && target.starts_with("/run")
            && let Some(parent) = target.parent()
        {
            cmd.arg("--ro-bind-try").arg(parent).arg(parent);
        }

        // 4. Hardware acceleration (GPU & Audio) across Intel, AMD, NVIDIA, ALSA
        cmd.arg("--dev-bind-try").arg("/dev/dri").arg("/dev/dri");
        cmd.arg("--dev-bind-try").arg("/dev/kfd").arg("/dev/kfd");
        cmd.arg("--dev-bind-try").arg("/dev/snd").arg("/dev/snd");

        // NVIDIA GPU device nodes
        for node in &[
            "/dev/nvidiactl",
            "/dev/nvidia-modeset",
            "/dev/nvidia-uvm",
            "/dev/nvidia-uvm-tools",
        ] {
            if Path::new(node).exists() {
                cmd.arg("--dev-bind-try").arg(node).arg(node);
            }
        }
        for i in 0..8 {
            let node = format!("/dev/nvidia{}", i);
            if Path::new(&node).exists() {
                cmd.arg("--dev-bind-try").arg(&node).arg(&node);
            }
        }

        // 5. CA certificates and font cache across major distros (Ubuntu, Fedora, Arch, openSUSE, Alpine)
        cmd.arg("--ro-bind-try").arg("/var/cache/fontconfig").arg("/var/cache/fontconfig");
        cmd.arg("--ro-bind-try").arg("/var/lib/ca-certificates").arg("/var/lib/ca-certificates");
        cmd.arg("--ro-bind-try").arg("/etc/ssl").arg("/etc/ssl");
        cmd.arg("--ro-bind-try").arg("/etc/pki").arg("/etc/pki");
        cmd.arg("--ro-bind-try").arg("/etc/ca-certificates").arg("/etc/ca-certificates");
        cmd.arg("--ro-bind-try").arg("/var/lib/dbus/machine-id").arg("/var/lib/dbus/machine-id");

        // 6. Network isolation
        if !allow_network {
            cmd.arg("--unshare-net");
        }

        // 7. GUI Display Forwarding (X11 & Wayland)
        cmd.arg("--ro-bind-try")
            .arg("/tmp/.X11-unix")
            .arg("/tmp/.X11-unix");

        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.arg("--ro-bind-try").arg(&runtime_dir).arg(&runtime_dir);
        }

        if let Ok(xauth) = std::env::var("XAUTHORITY") {
            cmd.arg("--ro-bind-try").arg(&xauth).arg(&xauth);
        }

        // 8. D-Bus session bus socket if specified outside XDG_RUNTIME_DIR
        if let Ok(dbus_addr) = std::env::var("DBUS_SESSION_BUS_ADDRESS")
            && let Some(path_str) = dbus_addr.strip_prefix("unix:path=")
        {
            let bus_path = path_str.split(',').next().unwrap_or(path_str);
            if Path::new(bus_path).exists() {
                cmd.arg("--ro-bind-try").arg(bus_path).arg(bus_path);
            }
        }

        // 9. Home directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        cmd.arg("--bind").arg(&home).arg(&home);

        let legacy_xauth = Path::new(&home).join(".Xauthority");
        if legacy_xauth.exists() {
            cmd.arg("--ro-bind-try").arg(&legacy_xauth).arg(&legacy_xauth);
        }

        // 10. Mount the binary itself read-only executable
        cmd.arg("--ro-bind").arg(binary_path).arg(binary_path);

        // 11. Environment variable: Force AppImage extraction inside unprivileged sandbox
        // Since FUSE cannot mount inside standard unprivileged user namespaces
        cmd.arg("--setenv").arg("APPIMAGE_EXTRACT_AND_RUN").arg("1");

        // 12. New PID namespace
        cmd.arg("--die-with-parent");

        // 13. The executable AppImage and its CLI arguments
        cmd.arg(binary_path);
        for a in args {
            cmd.arg(a);
        }

        let status = cmd
            .status()
            .map_err(|e| CartridgeError::Sandbox(format!("Failed to spawn sandbox: {e}")))?;

        Ok(status)
    }
}
