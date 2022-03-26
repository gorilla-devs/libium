use clap::ArgEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    /// The index of the active profile
    pub active_profile: usize,
    /// The profiles
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    /// The profile's name
    pub name: String,
    /// The directory to download mod JARs to
    pub output_dir: PathBuf,
    /// Check if mod JARs are compatible with this Minecraft version
    pub game_version: String,
    /// Check if mod JARs are compatible with this mod loader
    pub mod_loader: ModLoaders,
    /// A list of all the mods configured
    pub mods: Vec<Mod>,
}

/// A mod, which can be from 3 different sources
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum Mod {
    CurseForgeProject {
        name: String,
        project_id: i32,
    },
    ModrinthProject {
        name: String,
        project_id: String,
    },
    GitHubRepository {
        name: String,
        full_name: (String, String),
    },
}

impl Mod {
    pub fn name(&self) -> &str {
        match self {
            Mod::CurseForgeProject { name, .. }
            | Mod::ModrinthProject { name, .. }
            | Mod::GitHubRepository { name, .. } => name,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ArgEnum)]
pub enum ModLoaders {
    Fabric,
    Forge,
}

impl std::fmt::Display for ModLoaders {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
