pub fn current_arch() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    {
        "x86_64"
    }
    #[cfg(target_arch = "aarch64")]
    {
        "aarch64"
    }
    #[cfg(target_arch = "arm")]
    {
        "armhf"
    }
    #[cfg(target_arch = "x86")]
    {
        "i686"
    }
    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "x86"
    )))]
    {
        std::env::consts::ARCH
    }
}

/// Checks if an asset name matches the target architecture.
/// Returns true if it clearly matches or if no conflicting arch is indicated.
pub fn matches_arch(filename: &str, target_arch: &str) -> bool {
    let lower = filename.to_lowercase();
    if !lower.ends_with(".appimage") {
        return false;
    }

    // Exclude zsync / digests / asc
    if lower.ends_with(".zsync")
        || lower.ends_with(".sha256")
        || lower.ends_with(".asc")
        || lower.ends_with(".sig")
    {
        return false;
    }

    match target_arch {
        "x86_64" => {
            // Must not be aarch64/arm64 or i386
            if lower.contains("aarch64")
                || lower.contains("arm64")
                || lower.contains("armv7")
                || lower.contains("armhf")
                || lower.contains("i686")
                || lower.contains("i386")
            {
                return false;
            }
            // If it explicitly states x86_64 / amd64 / x64, that's a positive match
            if lower.contains("x86_64")
                || lower.contains("amd64")
                || lower.contains("x64")
                || lower.contains("x86-64")
            {
                return true;
            }
            // If it doesn't mention arch at all (e.g. app-1.0.AppImage), x86_64 is the primary default
            true
        }
        "aarch64" => {
            if lower.contains("x86_64") || lower.contains("amd64") || lower.contains("x64") {
                return false;
            }
            lower.contains("aarch64") || lower.contains("arm64") || lower.contains("armv8")
        }
        "armhf" => lower.contains("armhf") || lower.contains("armv7") || lower.contains("armv7l"),
        "i686" => lower.contains("i686") || lower.contains("i386") || lower.contains("x86"),
        _ => true,
    }
}

/// Score asset candidate based on closeness of arch match and clean AppImage suffix
pub fn asset_score(filename: &str, target_arch: &str) -> u32 {
    let lower = filename.to_lowercase();
    if !lower.ends_with(".appimage") {
        return 0;
    }
    let mut score: u32 = 10;
    if target_arch == "x86_64" {
        if lower.contains("x86_64") {
            score += 50;
        } else if lower.contains("amd64") {
            score += 45;
        } else if lower.contains("x64") {
            score += 40;
        }
    } else if target_arch == "aarch64" {
        if lower.contains("aarch64") {
            score += 50;
        } else if lower.contains("arm64") {
            score += 45;
        }
    }

    // Prefer standard release builds over continuous/nightly if unspecified
    if lower.contains("continuous") || lower.contains("nightly") {
        score = score.saturating_sub(5);
    }
    score
}
