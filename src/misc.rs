use crate::HOME;
use ferinth::Ferinth;
use std::path::PathBuf;

// macOS can only use a sync file picker
#[cfg(all(target_os = "macos", feature = "gui"))]
#[allow(clippy::unused_async)]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &PathBuf) -> Option<PathBuf> {
    rfd::FileDialog::new().set_directory(path).pick_folder()
}

// Other OSs can use the async version
#[cfg(all(not(target_os = "macos"), feature = "gui"))]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &PathBuf) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(path)
        .pick_folder()
        .await
        .map(|handle| handle.path().into())
}

#[cfg(not(feature = "gui"))]
#[allow(clippy::unused_async)]
pub async fn pick_folder(_: &PathBuf) -> Option<PathBuf> {
    use dialoguer::{theme::ColorfulTheme, Input};

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a mod output directory")
        .interact()
        .ok()?;
    Some(input.into())
}

/// Get a maximum of `count` number of the latest versions of Minecraft from the `version_manifest` provided
pub async fn get_latest_mc_versions(mut count: usize) -> Result<Vec<String>, ferinth::Error> {
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

/// Remove the given semver `input`'s patch version
pub fn remove_semver_patch(input: &str) -> Result<String, semver::Error> {
    // If the input string contains only one period, it already doesn't have the patch version
    if input.matches('.').count() == 1 {
        // So directly return the string
        Ok(input.into())
    } else {
        // Or else parse the string
        let version = semver::Version::parse(input)?;
        // And return the major and minor versions
        Ok(format!("{}.{}", version.major, version.minor))
    }
}

/// Get the Minecraft mods directory based on the current OS
/// If the OS doesn't match "macos", "linux", or "windows", this function will panic
pub fn get_mods_dir() -> PathBuf {
    match std::env::consts::OS {
        "macos" => HOME
            .join("Library")
            .join("ApplicationSupport")
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
