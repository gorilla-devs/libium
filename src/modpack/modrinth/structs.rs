use ferinth::structures::project_structs::ProjectSupportRange;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    /// The version of the format, stored as a number.
    /// The current value at the time of writing is `1`
    pub format_version: u64,
    /// The game of the modpack
    pub game: Game,
    /// A unique identifier for this specific version of the modpack
    pub version_id: String,
    /// Human readable name of the modpack
    pub name: String,
    /// A short description of this modpack.
    pub summary: Option<String>,
    /// A list of files for the modpack that needs to be downloaded
    pub files: Vec<ModpackFile>,
    /// A list of IDs and version numbers that launchers will use in order to know what to install
    pub dependencies: HashMap<DependencyID, String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
#[serde(deny_unknown_fields)]
pub enum DependencyID {
    Minecraft,
    Forge,
    FabricLoader,
    QuiltLoader,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ModpackFile {
    /// The destination path of this file, relative to the Minecraft instance directory
    pub path: PathBuf,
    /// The hashes of the file specified
    pub hashes: ModpackFileHashes,
    /// The  specific environment this file exists on
    pub env: Option<ModpackFileEnvironment>,
    /// HTTPS URLs where this file may be downloaded
    pub downloads: Vec<Url>,
    /// The size of the file in bytes
    pub file_size: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ModpackFileHashes {
    sha1: String,
    sha512: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ModpackFileEnvironment {
    client: ProjectSupportRange,
    server: ProjectSupportRange,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum Game {
    Minecraft,
}
