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
    match std::env::consts::OS {
        "macos" => HOME
            .join("Library")
            .join("Application Support")
            .join("minecraft"),
        "linux" => HOME.join(".minecraft"),
        "windows" => HOME.join("AppData").join("Roaming").join(".minecraft"),
        "android" => PathBuf::from("/")
            .join("storage")
            .join("emulated")
            .join("0")
            .join("games")
            .join("PojavLauncher")
            .join(".minecraft"),
        _ => panic!("Unsupported OS"),
    }
}
