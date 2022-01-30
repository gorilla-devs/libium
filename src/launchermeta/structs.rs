use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct VersionManifestV2 {
    /// IDs of the latest versions of Minecraft
    pub latest: LatestVersions,
    /// All versions of Minecraft
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LatestVersions {
    /// ID of the latest release
    pub release: String,
    /// ID of the latest snapshot
    pub snapshot: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    /// ID of the version
    pub id: String,
    #[serde(rename = "type")]
    /// Type of version
    pub version_type: VersionType,
    /// URL to version's manifest
    pub url: String,
    pub time: String,
    /// Time when the version was released
    pub release_time: String,
    /// SHA1 hash of the version
    pub sha1: String,
    /// Whether this version is "historical"
    pub compliance_level: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum VersionType {
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "old_beta")]
    Beta,
    #[serde(rename = "old_alpha")]
    Alpha,
}
