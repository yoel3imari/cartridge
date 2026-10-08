use std::path::Path;

pub struct DesktopEntryMutator;

impl DesktopEntryMutator {
    /// Extract application name from raw desktop file if present
    pub fn extract_name_from_desktop(raw_content: &str) -> Option<String> {
        let mut in_desktop_entry = false;
        for line in raw_content.lines() {
            let trimmed = line.trim();
            if trimmed == "[Desktop Entry]" {
                in_desktop_entry = true;
                continue;
            } else if trimmed.starts_with('[') {
                in_desktop_entry = false;
            }

            if in_desktop_entry && trimmed.starts_with("Name=") && !trimmed.starts_with("Name[") {
                let name = trimmed.trim_start_matches("Name=").trim();
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    /// Extract argument tail from an Exec= line (e.g. "Exec=foo %u" -> "%U", "Exec=foo --flag %F" -> "--flag %F")
    fn extract_exec_args(exec_line: &str) -> Option<String> {
        let line = exec_line.trim_start_matches("Exec=").trim();
        if line.is_empty() {
            return None;
        }

        let rest = if line.starts_with('"') {
            if let Some(end_quote) = line[1..].find('"') {
                line[end_quote + 2..].trim()
            } else {
                ""
            }
        } else if let Some(space_idx) = line.find(|c: char| c.is_whitespace()) {
            line[space_idx..].trim()
        } else {
            ""
        };

        if rest.is_empty() {
            None
        } else {
            // Modern desktop environments (GNOME, KDE) standardize on %U for URL/file lists
            let normalized = if rest == "%u" { "%U" } else { rest };
            Some(normalized.to_string())
        }
    }

    /// Mutate an existing desktop entry content or generate a new one
    pub fn build_desktop_entry(
        raw_content: Option<&str>,
        app_id: &str,
        app_name: &str,
        exec_path: &Path,
        icon_entry: &str,
        categories: &[String],
        description: Option<&str>,
    ) -> String {
        // If FUSE is not available on host, run through extraction mode fallback
        let base_exec = if !crate::util::fuse::is_fuse_available()
            && std::env::var("CARTRIDGE_FORCE_FUSE").is_err()
        {
            format!("env APPIMAGE_EXTRACT_AND_RUN=1 \"{}\"", exec_path.display())
        } else {
            format!("\"{}\"", exec_path.display())
        };

        let default_exec_str = format!("{} %U", base_exec);
        // TryExec MUST NOT be quoted per FreeDesktop desktop entry specification
        let try_exec_str = format!("{}", exec_path.display());

        if let Some(raw) = raw_content {
            let mut lines = Vec::new();
            let mut has_exec = false;
            let mut has_try_exec = false;
            let mut has_icon = false;
            let mut has_name = false;
            let mut has_startup_wm_class = false;
            let mut has_startup_notify = false;
            let mut in_desktop_entry_section = false;
            let mut in_action_section = false;

            for line in raw.lines() {
                let trimmed = line.trim();
                if trimmed == "[Desktop Entry]" {
                    in_desktop_entry_section = true;
                    in_action_section = false;
                    lines.push(line.to_string());
                    continue;
                } else if trimmed.starts_with("[Desktop Action ") {
                    in_desktop_entry_section = false;
                    in_action_section = true;
                    lines.push(line.to_string());
                    continue;
                } else if trimmed.starts_with('[') {
                    in_desktop_entry_section = false;
                    in_action_section = false;
                }

                if in_desktop_entry_section {
                    if trimmed.starts_with("Exec=") {
                        let final_exec = if let Some(args) = Self::extract_exec_args(trimmed) {
                            format!("{} {}", base_exec, args)
                        } else {
                            default_exec_str.clone()
                        };
                        lines.push(format!("Exec={}", final_exec));
                        has_exec = true;
                        continue;
                    } else if trimmed.starts_with("TryExec=") {
                        lines.push(format!("TryExec={}", try_exec_str));
                        has_try_exec = true;
                        continue;
                    } else if trimmed.starts_with("Icon=") {
                        lines.push(format!("Icon={}", icon_entry));
                        has_icon = true;
                        continue;
                    } else if trimmed.starts_with("Name=") && !trimmed.starts_with("Name[") {
                        lines.push(line.to_string());
                        has_name = true;
                        continue;
                    } else if trimmed.starts_with("StartupWMClass=") {
                        has_startup_wm_class = true;
                        lines.push(line.to_string());
                        continue;
                    } else if trimmed.starts_with("StartupNotify=") {
                        has_startup_notify = true;
                        lines.push(line.to_string());
                        continue;
                    }
                } else if in_action_section && trimmed.starts_with("Exec=") {
                    let action_exec = if let Some(args) = Self::extract_exec_args(trimmed) {
                        format!("{} {}", base_exec, args)
                    } else {
                        base_exec.clone()
                    };
                    lines.push(format!("Exec={}", action_exec));
                    continue;
                }

                lines.push(line.to_string());
            }

            // Append missing essential keys to [Desktop Entry] section
            let mut result = Vec::new();
            for line in lines {
                result.push(line.clone());
                if line.trim() == "[Desktop Entry]" {
                    if !has_exec {
                        result.push(format!("Exec={}", default_exec_str));
                    }
                    if !has_try_exec {
                        result.push(format!("TryExec={}", try_exec_str));
                    }
                    if !has_icon {
                        result.push(format!("Icon={}", icon_entry));
                    }
                    if !has_name {
                        result.push(format!("Name={}", app_name));
                    }
                    if !has_startup_wm_class {
                        result.push(format!("StartupWMClass={}", app_id));
                    }
                    if !has_startup_notify {
                        result.push("StartupNotify=true".to_string());
                    }
                }
            }

            let mut out = result.join("\n");
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out
        } else {
            // Generate minimal standard .desktop file compliant with FreeDesktop, GNOME, and KDE
            let cat_str = if categories.is_empty() {
                "Utility;".to_string()
            } else {
                let mut c = categories.join(";");
                if !c.ends_with(';') {
                    c.push(';');
                }
                c
            };

            let comment = description.unwrap_or(app_name);

            format!(
                "[Desktop Entry]\n\
                 Type=Application\n\
                 Version=1.0\n\
                 Name={app_name}\n\
                 Comment={comment}\n\
                 Exec={default_exec_str}\n\
                 TryExec={try_exec_str}\n\
                 Icon={icon_entry}\n\
                 Terminal=false\n\
                 Categories={cat_str}\n\
                 StartupWMClass={app_id}\n\
                 StartupNotify=true\n"
            )
        }
    }
}
