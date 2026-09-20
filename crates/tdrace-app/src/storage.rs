use std::fs;
use std::path::PathBuf;

/// Environment variable to override the root user data directory (e.g. in tests or portable builds).
pub const ENV_USER_DATA_DIR: &str = "TDRACE_USER_DATA_DIR";

/// Environment variable to override the user configuration directory (e.g. in tests or portable builds).
pub const ENV_USER_CONFIG_DIR: &str = "TDRACE_USER_CONFIG_DIR";

/// Environment variable to override the user circuits/tracks directory.
pub const ENV_USER_TRACKS_DIR: &str = "TDRACE_USER_TRACKS_DIR";

/// Environment variable to override the git tracks directory (e.g. in tests).
pub const ENV_GIT_TRACKS_DIR: &str = "TDRACE_GIT_TRACKS_DIR";

/// Environment variable indicating developer mode execution.
pub const ENV_DEV_MODE: &str = "TDRACE_DEV";

/// Checks whether developer mode is active.
///
/// Returns `true` if:
/// 1. `TDRACE_DEV` environment variable is set to "1" or "true" (case-insensitive)
/// 2. Any CLI argument equals `--dev` or `-d`
pub fn is_dev_mode() -> bool {
    if let Ok(val) = std::env::var(ENV_DEV_MODE) {
        if val == "1" || val.eq_ignore_ascii_case("true") {
            return true;
        }
    }
    std::env::args().any(|arg| arg == "--dev" || arg == "-d")
}

/// Environment variable indicating automated test mode execution.
pub const ENV_TEST_MODE: &str = "TDRACE_TEST_MODE";

/// Checks whether execution is currently running within an automated test runner (e.g. `cargo test`, `nextest`).
///
/// Ensures tests NEVER inadvertently access or mutate live host user configuration or data directories.
pub fn is_test_environment() -> bool {
    // 1. Explicit override environment variable
    if let Ok(val) = std::env::var(ENV_TEST_MODE) {
        if val == "1" || val.eq_ignore_ascii_case("true") {
            return true;
        }
        if val == "0" || val.eq_ignore_ascii_case("false") {
            return false;
        }
    }
    // 2. Cargo test / nextest environment variables
    if std::env::var("NEXTEST").is_ok() || std::env::var("CARGO_TARGET_TMPDIR").is_ok() {
        return true;
    }
    // 3. Inspect executable path: test runners are placed in target/.../deps/
    if let Ok(exe) = std::env::current_exe() {
        let exe_str = exe.to_string_lossy();
        if exe_str.contains("/deps/") || exe_str.contains("\\deps\\") {
            return true;
        }
    }
    // 4. Test harness command-line arguments
    if std::env::args().any(|a| a == "--test" || a == "--nocapture" || a == "--bench" || a == "--exact") {
        return true;
    }
    false
}

/// Resolves the repository's git-tracked `tracks/` directory when running in dev mode.
/// Checks current working directory (`tracks`), parent directory, or relative paths.
pub fn resolve_git_tracks_dir() -> Option<PathBuf> {
    if let Ok(val) = std::env::var(ENV_GIT_TRACKS_DIR) {
        if !val.trim().is_empty() {
            let p = PathBuf::from(val);
            if p.is_dir() {
                return p.canonicalize().ok().or_else(|| Some(p));
            }
        }
    }
    let candidates = [
        PathBuf::from("tracks"),
        PathBuf::from("../tracks"),
        PathBuf::from("../../tracks"),
    ];
    for c in &candidates {
        if c.is_dir() {
            let parent = c.parent().unwrap_or(std::path::Path::new("."));
            if parent.join("crates").is_dir() || parent.join(".git").exists() {
                return c.canonicalize().ok().or_else(|| Some(c.clone()));
            }
        }
    }
    None
}

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

    // Safety guard: Automated tests must NEVER mutate the host user's live game data directory!
    if is_test_environment() {
        let test_sandbox = std::env::temp_dir().join("tdrace_test_sandbox").join("data");
        let _ = fs::create_dir_all(&test_sandbox);
        return test_sandbox;
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

/// Resolves the user-specific configuration root directory.
///
/// Priority order:
/// 1. `TDRACE_USER_CONFIG_DIR` environment variable
/// 2. If `TDRACE_USER_DATA_DIR` is set, use it (ensures test harnesses isolating data also isolate config)
/// 3. Platform-specific user config directory:
///    - Linux / BSD: `$XDG_CONFIG_HOME/tdrace` (or `~/.config/tdrace`)
///    - macOS: `~/Library/Application Support/tdrace`
///    - Windows: `%APPDATA%\tdrace` (or `%LOCALAPPDATA%\tdrace`)
/// 4. Fallback: System temp directory (`<temp_dir>/tdrace`)
pub fn resolve_user_config_dir() -> PathBuf {
    if let Ok(override_dir) = std::env::var(ENV_USER_CONFIG_DIR) {
        if !override_dir.trim().is_empty() {
            let p = PathBuf::from(override_dir);
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    if let Ok(override_data) = std::env::var(ENV_USER_DATA_DIR) {
        if !override_data.trim().is_empty() {
            let p = PathBuf::from(override_data);
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    // Safety guard: Automated tests must NEVER mutate the host user's live game configuration directory!
    if is_test_environment() {
        let test_sandbox = std::env::temp_dir().join("tdrace_test_sandbox").join("config");
        let _ = fs::create_dir_all(&test_sandbox);
        return test_sandbox;
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
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg.trim().is_empty() {
                let p = PathBuf::from(xdg).join("tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                let p = PathBuf::from(home).join(".config").join("tdrace");
                let _ = fs::create_dir_all(&p);
                return p;
            }
        }
    }

    let fallback = std::env::temp_dir().join("tdrace");
    let _ = fs::create_dir_all(&fallback);
    fallback
}

/// Resolves the user-specific `config.toml` path (`<resolve_user_config_dir()>/config.toml`).
pub fn resolve_user_config_path() -> PathBuf {
    resolve_user_config_dir().join("config.toml")
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

/// Loads custom input mappings from `<user_data_dir>/input_bindings.json`, if present.
pub fn load_input_bindings() -> Option<cabinet::input::mapping::InputMap> {
    let path = resolve_user_data_dir().join("input_bindings.json");
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(map) = cabinet::input::mapping::InputMap::from_json(&content) {
                return Some(map);
            }
        }
    }
    None
}

/// Saves custom input mappings to `<user_data_dir>/input_bindings.json`.
pub fn save_input_bindings(map: &cabinet::input::mapping::InputMap) -> Result<(), std::io::Error> {
    let dir = resolve_user_data_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("input_bindings.json");
    let json = map.to_json().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

/// Global mutex for tests that override environment variables (such as `TDRACE_USER_CONFIG_DIR`).
pub static ENV_CONFIG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// RAII guard that temporarily isolates `TDRACE_USER_CONFIG_DIR` to a temporary directory for tests,
/// preventing any test from mutating the developer or player's personal `~/.config/tdrace/config.toml`.
pub struct ScopedTempConfigDir {
    path: PathBuf,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl ScopedTempConfigDir {
    pub fn new(prefix: &str) -> Self {
        let guard = ENV_CONFIG_MUTEX.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let path = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&path);
        std::env::set_var(ENV_USER_CONFIG_DIR, &path);
        Self { path, _guard: guard }
    }

    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for ScopedTempConfigDir {
    fn drop(&mut self) {
        std::env::remove_var(ENV_USER_CONFIG_DIR);
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static STORAGE_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_resolve_user_tracks_dir_env_override() {
        let _guard = STORAGE_TEST_MUTEX.lock().unwrap();
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

    #[test]
    fn test_resolve_user_config_dir_env_override() {
        let _guard = STORAGE_TEST_MUTEX.lock().unwrap();
        let temp = std::env::temp_dir().join(format!(
            "tdrace_test_config_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&temp);

        std::env::set_var(ENV_USER_CONFIG_DIR, &temp);
        let resolved_dir = resolve_user_config_dir();
        assert_eq!(resolved_dir, temp);
        assert!(resolved_dir.exists());

        let resolved_file = resolve_user_config_path();
        assert_eq!(resolved_file, temp.join("config.toml"));

        std::env::remove_var(ENV_USER_CONFIG_DIR);
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn test_is_dev_mode_detection() {
        let _guard = STORAGE_TEST_MUTEX.lock().unwrap();
        std::env::remove_var(ENV_DEV_MODE);
        // Defaults to false when env var is not set and no --dev arg
        assert!(!is_dev_mode());

        std::env::set_var(ENV_DEV_MODE, "1");
        assert!(is_dev_mode());

        std::env::set_var(ENV_DEV_MODE, "true");
        assert!(is_dev_mode());

        std::env::remove_var(ENV_DEV_MODE);
    }

    #[test]
    fn test_is_test_environment_detection() {
        let _guard = STORAGE_TEST_MUTEX.lock().unwrap();
        // Since this test runs as a cargo test binary, is_test_environment() must detect it automatically
        assert!(is_test_environment());

        // Explicit override: false
        std::env::set_var(ENV_TEST_MODE, "false");
        assert!(!is_test_environment());

        // Explicit override: true
        std::env::set_var(ENV_TEST_MODE, "true");
        assert!(is_test_environment());

        std::env::remove_var(ENV_TEST_MODE);
    }

    #[test]
    fn test_test_environment_automatically_sandboxes_directories() {
        let _guard = STORAGE_TEST_MUTEX.lock().unwrap();
        std::env::remove_var(ENV_USER_CONFIG_DIR);
        std::env::remove_var(ENV_USER_DATA_DIR);

        assert!(is_test_environment());
        let config_dir = resolve_user_config_dir();
        let data_dir = resolve_user_data_dir();

        // Must resolve into temp sandbox, never host user dirs
        assert!(config_dir.starts_with(std::env::temp_dir()));
        assert!(data_dir.starts_with(std::env::temp_dir()));
        assert!(config_dir.to_string_lossy().contains("tdrace_test_sandbox"));
        assert!(data_dir.to_string_lossy().contains("tdrace_test_sandbox"));

        if let Ok(home) = std::env::var("HOME") {
            if !home.trim().is_empty() {
                let real_config = PathBuf::from(&home).join(".config").join("tdrace");
                let real_data = PathBuf::from(&home).join(".local").join("share").join("tdrace");
                assert_ne!(config_dir, real_config);
                assert_ne!(data_dir, real_data);
            }
        }
    }
}
