pub mod add;
pub mod config;
pub mod file_picker;
pub mod modpack;
pub mod upgrade;
pub mod version_ext;

use once_cell::sync::Lazy;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, Result};

pub static HOME: Lazy<PathBuf> =
    Lazy::new(|| home::home_dir().expect("Could not get user's home directory"));

/// Get the default Minecraft instance directory based on the current compilation `target_os`.
/// If the `target_os` doesn't match `"macos"`, `"linux"`, or `"windows"`, this function will not compile.
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
}

/// Read `source` and return the data as a string
///
/// A wrapper for dealing with the read buffer.
pub async fn read_wrapper(mut source: impl AsyncReadExt + Unpin) -> Result<String> {
    let mut buffer = String::new();
    source.read_to_string(&mut buffer).await?;
    Ok(buffer)
}

pub struct APIs<'a> {
    pub mr: &'a ferinth::Ferinth,
    pub cf: &'a furse::Furse,
    pub gh: &'a octocrab::Octocrab,
}

impl<'a> APIs<'a> {
    pub fn new(mr: &'a ferinth::Ferinth, cf: &'a furse::Furse, gh: &'a octocrab::Octocrab) -> Self {
        Self { mr, cf, gh }
    }
}
