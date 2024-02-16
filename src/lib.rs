pub mod add;
pub mod config;
pub mod file_picker;
pub mod modpack;
pub mod upgrade;
pub mod version_ext;

use once_cell::sync::Lazy;
use std::path::PathBuf;

pub static HOME: Lazy<PathBuf> =
    Lazy::new(|| home::home_dir().expect("Could not get user's home directory"));

/// Get the default Minecraft instance directory based on the current OS.
/// If the OS doesn't match `"macos"`, `"linux"`, `"windows"`, or `"android"`, this function will panic.
pub fn get_minecraft_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    return HOME.join("AppData").join("Roaming").join(".minecraft");

    #[cfg(target_os = "macos")]
    return HOME
        .join("Library")
        .join("Application Support")
        .join("minecraft");

    #[cfg(target_os = "linux")]
    return HOME.join(".minecraft");

    #[cfg(target_os = "android")]
    return PathBuf::from("/")
        .join("storage")
        .join("emulated")
        .join("0")
        .join("games")
        .join("PojavLauncher")
        .join(".minecraft");
}
