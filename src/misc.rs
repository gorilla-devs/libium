use crate::HOME;
use ferinth::Ferinth;
use std::path::PathBuf;

/// Get a maximum of `count` number of the latest major versions of Minecraft
pub async fn get_major_mc_versions(mut count: usize) -> Result<Vec<String>, ferinth::Error> {
    let versions = Ferinth::default().list_game_versions().await?;
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
