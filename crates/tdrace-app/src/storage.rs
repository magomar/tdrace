use std::fs;
use std::path::PathBuf;

/// Environment variable to override the root user data directory (e.g. in tests or portable builds).
pub const ENV_USER_DATA_DIR: &str = "TDRACE_USER_DATA_DIR";

/// Environment variable to override the user circuits/tracks directory.
pub const ENV_USER_TRACKS_DIR: &str = "TDRACE_USER_TRACKS_DIR";

/// Resolves the user-specific storage root directory for local game data, profiles, and tracks.
///
/// Priority order:
/// 1. `TDRACE_USER_DATA_DIR` environment variable
/// 2. Platform-specific user data directory:
///    - Linux / BSD: `$XDG_DATA_HOME/tdrace` (or `~/.local/share/tdrace`)
///    - macOS: `~/Library/Application Support/tdrace`
///    - Windows: `%APPDATA%\tdrace` (or `%LOCALAPPDATA%\tdrace`)
/// 3. Fallback: System temp directory (`<temp_dir>/tdrace`)
pub fn resolve_user_data_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var(ENV_USER_DATA_DIR) {
        if !override_dir.trim().is_empty() {
            let p = PathBuf::from(override_dir);
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(app_data) = std::env::var("APPDATA") {
            if !app_data.trim().is_empty() {
                let p = PathBuf::from(app_data).join("tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            if !local_app_data.trim().is_empty() {
                let p = PathBuf::from(local_app_data).join("tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                let p = PathBuf::from(home).join("Library/Application Support/tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            if !xdg.trim().is_empty() {
                let p = PathBuf::from(xdg).join("tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                let p = PathBuf::from(home).join(".local/share/tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
    }

    let fallback = std::env::temp_dir().join("tdrace");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

/// Resolves the user-specific circuits/tracks directory (`<user_data_dir>/tracks`).
///
/// Priority order:
/// 1. `TDRACE_USER_TRACKS_DIR` environment variable
/// 2. `<resolve_user_data_dir()>/tracks`
pub fn resolve_user_tracks_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var(ENV_USER_TRACKS_DIR) {
        if !override_dir.trim().is_empty() {
            let p = PathBuf::from(override_dir);
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    let p = resolve_user_data_dir().join("tracks");
    let _ = fs::create_dir_all(&p);
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_user_tracks_dir_env_override() {
        let temp = std::env::temp_dir().join(format!(
            "tdrace_test_storage_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp);

        std::env::set_var(ENV_USER_TRACKS_DIR, &temp);
        let resolved = resolve_user_tracks_dir();
        assert_eq!(resolved, temp);
        assert!(resolved.exists());

        std::env::remove_var(ENV_USER_TRACKS_DIR);
        let _ = fs::remove_dir_all(&temp);
    }
}
