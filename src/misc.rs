use crate::{launchermeta, HOME};
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
pub fn get_latest_mc_versions(
    count: usize,
    version_manifest: launchermeta::structs::VersionManifestV2,
) -> Result<Vec<String>, semver::Error> {
    let versions = version_manifest.versions;
    let mut versions_to_display: Vec<String> = Vec::new();
    let mut major_versions_added: Vec<String> = Vec::new();

    for version in versions {
        if versions_to_display.len() >= count {
            break;
        }
        let major_version = if matches!(
            version.version_type,
            launchermeta::structs::VersionType::Release
        ) {
            remove_semver_patch(&version.id)?
        } else {
            continue;
        };

        // If version is a release and it hasn't already been added
        if matches!(
            version.version_type,
            launchermeta::structs::VersionType::Release
        ) && !major_versions_added.contains(&major_version)
        {
            versions_to_display.push(version.id);
            major_versions_added.push(major_version);
        }
    }

    Ok(versions_to_display)
}

/// Remove the given semver `input`'s patch version
///
/// ```rust
/// # use libium::misc::remove_semver_patch;
/// assert!(remove_semver_patch("1.7.10")? == "1.7".to_string());
/// assert!(remove_semver_patch("1.14.4")? == "1.14".to_string());
/// // Versions already without a minor version are preserved
/// assert!(remove_semver_patch("1.18")? == "1.18".to_string());
/// # Ok::<(), semver::Error>(())
/// ```
pub fn remove_semver_patch(input: &str) -> Result<String, semver::Error> {
    // If the input string contains only one period, it already doesn't have the patch version
    if input.matches('.').count() == 1 {
        // So directly return the string
        Ok(input.into())
    } else {
        // Or else parse the string in to a semver version struct
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
