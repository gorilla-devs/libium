//! Contains structure definitions for the configuration file0

use clap::ArgEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct Config {
    /// The index of the active profile
    pub active_profile: usize,
    /// The profiles
    pub profiles: Vec<Profile>,
}

#[derive(Deserialize, Serialize)]
pub struct Profile {
    /// The profile's name
    pub name: String,
    /// The directory to download mod JARs to
    pub output_dir: PathBuf,
    /// Check if mod JARs are compatible with this Minecraft version
    pub game_version: String,
    /// Check if mod JARs are compatible with this mod loader
    pub mod_loader: ModLoaders,
    /// Project IDs of CurseForge mods
    pub curse_projects: Vec<i32>,
    /// Mod IDs of Modrinth mods
    pub modrinth_mods: Vec<String>,
    /// Full names of GitHub repositories
    pub github_repos: Vec<(String, String)>,
}

#[derive(ArgEnum, Clone, Deserialize, Serialize, Debug)]
pub enum ModLoaders {
    Fabric,
    Forge,
}

impl std::fmt::Display for ModLoaders {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
