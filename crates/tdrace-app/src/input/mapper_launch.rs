//! Launching the external `gamepad-mapper` calibration tool from the Controls settings.
//!
//! The mapper is a separate program (its own window and license). It saves
//! `gamepad_profile.json` to its working directory and to `~/.config/gamepad-mapper/`, both of
//! which [`GamepadController`](crate::input::gamepad::GamepadController) already searches, so
//! TDRace only has to find the binary, start it, and reload the profile when it exits.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Environment variable with an explicit path to the gamepad-mapper executable.
pub const ENV_GAMEPAD_MAPPER: &str = "TDRACE_GAMEPAD_MAPPER";

/// File name of the mapper executable on this platform.
pub fn mapper_file_name() -> String {
    format!("gamepad-mapper{}", std::env::consts::EXE_SUFFIX)
}

/// Finds the gamepad-mapper executable, in order:
/// 1. `TDRACE_GAMEPAD_MAPPER`,
/// 2. next to the TDRace executable (how a packaged build ships it),
/// 3. on `PATH`,
/// 4. a development build in a `gamepad-mapper` checkout beside the working directory or one of
///    its parents (`gamepad-mapper/target/{release,debug}/`).
pub fn find_gamepad_mapper() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let cwd = std::env::current_dir().unwrap_or_default();
    find_gamepad_mapper_in(
        std::env::var_os(ENV_GAMEPAD_MAPPER).as_deref(),
        exe_dir.as_deref(),
        std::env::var_os("PATH").as_deref(),
        &cwd,
    )
}

/// [`find_gamepad_mapper`] with every input explicit, for tests.
pub fn find_gamepad_mapper_in(
    env_override: Option<&OsStr>,
    exe_dir: Option<&Path>,
    path_var: Option<&OsStr>,
    cwd: &Path,
) -> Option<PathBuf> {
    let name = mapper_file_name();
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(explicit) = env_override.filter(|v| !v.is_empty()) {
        candidates.push(PathBuf::from(explicit));
    }
    if let Some(dir) = exe_dir {
        candidates.push(dir.join(&name));
    }
    if let Some(path) = path_var {
        candidates.extend(std::env::split_paths(path).map(|dir| dir.join(&name)));
    }
    for ancestor in cwd.ancestors().take(6) {
        for profile in ["release", "debug"] {
            candidates.push(ancestor.join("gamepad-mapper").join("target").join(profile).join(&name));
        }
    }
    candidates.into_iter().find(|p| p.is_file())
}

/// A running gamepad-mapper process.
#[derive(Debug)]
pub struct GamepadMapperProcess {
    child: std::process::Child,
}

impl GamepadMapperProcess {
    /// Starts the mapper. `working_dir` is where it writes its local `gamepad_profile.json`.
    pub fn launch(executable: &Path, working_dir: &Path) -> std::io::Result<Self> {
        let _ = std::fs::create_dir_all(working_dir);
        let child = std::process::Command::new(executable).current_dir(working_dir).spawn()?;
        Ok(Self { child })
    }

    /// True once the mapper window has been closed.
    pub fn has_exited(&mut self) -> bool {
        !matches!(self.child.try_wait(), Ok(None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tdrace_mapper_test_{}_{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"").unwrap();
    }

    #[test]
    fn explicit_env_path_wins() {
        let root = temp_dir("env");
        let explicit = root.join("custom").join("my-mapper");
        touch(&explicit);
        touch(&root.join("bin").join(mapper_file_name()));

        let found = find_gamepad_mapper_in(Some(explicit.as_os_str()), Some(&root.join("bin")), None, &root);

        assert_eq!(found, Some(explicit));
    }

    #[test]
    fn packaged_binary_next_to_tdrace_is_found_before_path() {
        let root = temp_dir("exe");
        let beside = root.join("app").join(mapper_file_name());
        let on_path = root.join("path_bin").join(mapper_file_name());
        touch(&beside);
        touch(&on_path);
        let path_var = std::env::join_paths([root.join("path_bin")]).unwrap();

        let found = find_gamepad_mapper_in(None, Some(&root.join("app")), Some(&path_var), &root);

        assert_eq!(found, Some(beside));
    }

    #[test]
    fn development_checkout_beside_a_parent_directory_is_found() {
        // games/tdrace/.claude/worktrees/x  ->  games/gamepad-mapper/target/release/gamepad-mapper
        let root = temp_dir("dev");
        let cwd = root.join("tdrace").join(".claude").join("worktrees").join("x");
        std::fs::create_dir_all(&cwd).unwrap();
        let dev_build = root.join("gamepad-mapper").join("target").join("release").join(mapper_file_name());
        touch(&dev_build);

        assert_eq!(find_gamepad_mapper_in(None, None, None, &cwd), Some(dev_build));
    }

    #[cfg(unix)]
    #[test]
    fn launched_process_is_reported_closed_once_it_exits() {
        let dir = temp_dir("launch");
        let mut process = GamepadMapperProcess::launch(Path::new("/usr/bin/true"), &dir).expect("spawn /usr/bin/true");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !process.has_exited() {
            assert!(std::time::Instant::now() < deadline, "process never reported as exited");
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn missing_mapper_is_none() {
        let root = temp_dir("none");
        let empty_env = OsStr::new("");
        assert_eq!(find_gamepad_mapper_in(Some(empty_env), Some(&root), None, &root), None);
    }
}
