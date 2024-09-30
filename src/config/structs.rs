use super::filters::Filter;
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    /// The index of the active profile
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_profile: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub profiles: Vec<Profile>,

    /// The index of the active modpack
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_modpack: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub modpacks: Vec<Modpack>,
}

const fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    pub name: String,
    pub output_dir: PathBuf,
    pub install_overrides: bool,
    pub identifier: ModpackIdentifier,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModpackIdentifier {
    CurseForgeModpack(i32),
    ModrinthModpack(String),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub name: String,

    /// The directory to download mod files to
    pub output_dir: PathBuf,

    // There will be no filters when reading a v4 config
    #[serde(default)]
    pub filters: Vec<Filter>,

    pub mods: Vec<Mod>,

    // Kept for backwards compatibility reasons (i.e. migrating from a v4 config)
    #[serde(skip_serializing)]
    game_version: Option<String>,
    #[serde(skip_serializing)]
    mod_loader: Option<ModLoader>,
}

impl Profile {
    /// A simple contructor that automatically deals with converting to filters
    pub fn new(
        name: String,
        output_dir: PathBuf,
        game_versions: Vec<String>,
        mod_loader: ModLoader,
    ) -> Self {
        Self {
            name,
            output_dir,
            filters: vec![
                Filter::ModLoaderPrefer(match mod_loader {
                    ModLoader::Quilt => vec![ModLoader::Quilt, ModLoader::Fabric],
                    _ => vec![mod_loader],
                }),
                Filter::GameVersionStrict(game_versions),
            ],
            mods: vec![],
            game_version: None,
            mod_loader: None,
        }
    }

    /// Convert the v4 profile's `game_version` and `mod_loader` fields into filters
    pub(crate) fn backwards_compat(&mut self) {
        if let (Some(version), Some(loader)) = (self.game_version.take(), self.mod_loader.take()) {
            self.filters = vec![
                Filter::ModLoaderPrefer(match loader {
                    ModLoader::Quilt => vec![ModLoader::Quilt, ModLoader::Fabric],
                    _ => vec![loader],
                }),
                Filter::GameVersionStrict(vec![version]),
            ];
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub identifier: ModIdentifier,

    /// The specific version of the mod to download
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pin: Option<String>,

    /// Custom filters that apply only for this mod
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub filters: Vec<Filter>,

    /// Whether the filters specified above replace or apply with the profile's filters
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub override_filters: bool,
}

const fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository((String, String)),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModIdentifierRef<'a> {
    CurseForgeProject(i32),
    ModrinthProject(&'a str),
    GitHubRepository((&'a str, &'a str)),
}

impl ModIdentifier {
    pub fn as_ref(&self) -> ModIdentifierRef {
        match self {
            ModIdentifier::CurseForgeProject(v) => ModIdentifierRef::CurseForgeProject(*v),
            ModIdentifier::ModrinthProject(v) => ModIdentifierRef::ModrinthProject(v),
            ModIdentifier::GitHubRepository(v) => ModIdentifierRef::GitHubRepository((&v.0, &v.1)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Display, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ModLoader {
    Quilt,
    Fabric,
    Forge,
    #[clap(name = "neoforge")]
    NeoForge,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("The given string is not a recognised mod loader")]
pub struct ModLoaderParseError;

impl FromStr for ModLoader {
    type Err = ModLoaderParseError;

    // This implementation is case-insensitive
    fn from_str(from: &str) -> Result<Self, Self::Err> {
        match from.trim().to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            "neoforge" => Ok(Self::NeoForge),
            _ => Err(Self::Err {}),
        }
    }
}
