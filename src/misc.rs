use crate::HOME;
use ferinth::Ferinth;
use std::path::PathBuf;

/// Get a maximum of `count` number of the latest major versions of Minecraft
pub async fn get_major_mc_versions(mut count: usize) -> Result<Vec<String>, ferinth::Error> {
    let versions = Ferinth::new().list_game_versions().await?;
    let mut major_versions = Vec::new();

    for version in versions {
        if count == 0 {
            break;
        }
        if version.major {
            major_versions.push(version.version);
            count -= 1;
        }
    }

    Ok(major_versions)
}

/// Get the Minecraft mods directory based on the current OS
/// If the OS doesn't match "macos", "linux", or "windows", this function will panic
pub fn get_mods_dir() -> PathBuf {
    match std::env::consts::OS {
        "macos" => HOME
            .join("Library")
            .join("Application Support")
            .join("minecraft")
            .join("mods"),
        "linux" => HOME.join(".minecraft").join("mods"),
        "windows" => HOME
            .join("AppData")
            .join("Roaming")
            .join(".minecraft")
            .join("mods"),
        _ => unreachable!(),
    }
}
