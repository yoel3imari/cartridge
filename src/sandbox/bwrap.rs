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

        // Base read-only system binds
        cmd.arg("--ro-bind").arg("/usr").arg("/usr");
        cmd.arg("--ro-bind-try").arg("/lib").arg("/lib");
        cmd.arg("--ro-bind-try").arg("/lib64").arg("/lib64");
        cmd.arg("--ro-bind-try").arg("/bin").arg("/bin");
        cmd.arg("--ro-bind-try").arg("/sbin").arg("/sbin");
        cmd.arg("--ro-bind-try").arg("/etc").arg("/etc");
        cmd.arg("--ro-bind-try").arg("/opt").arg("/opt");

        // Dynamic system devices and runtime
        cmd.arg("--proc").arg("/proc");
        cmd.arg("--dev").arg("/dev");
        cmd.arg("--tmpfs").arg("/tmp");
        cmd.arg("--tmpfs").arg("/run");

        // Network isolation
        if !allow_network {
            cmd.arg("--unshare-net");
        }

        // GUI Display Forwarding (X11 & Wayland)
        cmd.arg("--ro-bind-try")
            .arg("/tmp/.X11-unix")
            .arg("/tmp/.X11-unix");

        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            cmd.arg("--ro-bind-try").arg(&runtime_dir).arg(&runtime_dir);
        }

        // Home directory - isolated tmpfs with mounted AppImage
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        cmd.arg("--bind").arg(&home).arg(&home);

        // Mount the binary itself read-only executable
        cmd.arg("--ro-bind").arg(binary_path).arg(binary_path);

        // New PID namespace
        cmd.arg("--die-with-parent");

        // The executable AppImage and its CLI arguments
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
