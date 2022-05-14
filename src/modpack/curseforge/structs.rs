use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Information about how to setup Minecraft
    pub minecraft: Minecraft,
    /// The type of this manifest ??
    pub manifest_type: ManifestType,
    /// The version of this manifest
    pub manifest_version: i32,
    /// The name of this modpack
    pub name: String,
    /// The version of this modpack
    pub version: String,
    /// The author who created this modpack
    pub author: String,
    /// The files this modpack needs
    pub files: Vec<ModpackFile>,
    /// A directory of overrides to install
    pub overrides: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Minecraft {
    /// The game version to install
    pub version: String,
    /// A list of mod loaders that can be used
    pub mod_loaders: Vec<ModpackModLoader>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub enum ManifestType {
    MinecraftModpack,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ModpackFile {
    #[serde(rename = "projectID")]
    /// The project ID of this mod
    pub project_id: i32,
    #[serde(rename = "fileID")]
    /// The specific file ID to download
    pub file_id: i32,
    /// Whether this mod is required
    pub required: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ModpackModLoader {
    /// The name/ID of the mod loader
    pub id: String,
    /// Whether this is the primary/recommended mod loader
    pub primary: bool,
}
