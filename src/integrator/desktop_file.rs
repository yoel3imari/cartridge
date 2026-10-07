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
        let exec_str = format!("\"{}\" %U", exec_path.display());
        // TryExec MUST NOT be quoted per FreeDesktop desktop entry specification
        let try_exec_str = format!("{}", exec_path.display());

        if let Some(raw) = raw_content {
            let mut lines = Vec::new();
            let mut has_exec = false;
            let mut has_try_exec = false;
            let mut has_icon = false;
            let mut has_name = false;
            let mut in_desktop_entry_section = false;

            for line in raw.lines() {
                let trimmed = line.trim();
                if trimmed == "[Desktop Entry]" {
                    in_desktop_entry_section = true;
                    lines.push(line.to_string());
                    continue;
                } else if trimmed.starts_with('[') {
                    in_desktop_entry_section = false;
                }

                if in_desktop_entry_section {
                    if trimmed.starts_with("Exec=") {
                        lines.push(format!("Exec={}", exec_str));
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
                    }
                }

                lines.push(line.to_string());
            }

            // Append missing essential keys to [Desktop Entry] section
            let mut result = Vec::new();
            for line in lines {
                result.push(line.clone());
                if line.trim() == "[Desktop Entry]" {
                    if !has_exec {
                        result.push(format!("Exec={}", exec_str));
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
                }
            }

            let mut out = result.join("\n");
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out
        } else {
            // Generate minimal standard .desktop file
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
                 Exec={exec_str}\n\
                 TryExec={try_exec_str}\n\
                 Icon={icon_entry}\n\
                 Terminal=false\n\
                 Categories={cat_str}\n\
                 StartupWMClass={app_id}\n"
            )
        }
    }
}
