pub mod structs;

use reqwest::{get, Result};

/// Get the version manifest v2 from Mojang
pub async fn get_version_manifest() -> Result<structs::VersionManifestV2> {
    get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .await?
        .json()
        .await
}
