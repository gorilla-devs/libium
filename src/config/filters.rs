use super::structs::ModLoader;
use crate::iter_ext::IterExt as _;
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};
use std::mem::discriminant;

#[derive(Deserialize, Serialize, Debug, Display, Clone)]
pub enum Filter {
    /// Prefers files in the order of the given loaders
    #[display("Mod Loader (Prefer): {}", _0.iter().display(", "))]
    ModLoaderPrefer(Vec<ModLoader>),

    /// Selects files that are compatible with any of the given loaders
    #[display("Mod Loader (Either): {}", _0.iter().display(", "))]
    ModLoaderAny(Vec<ModLoader>),

    /// Selects files strictly compatible with the versions specified
    #[display("Game Version: {}", _0.iter().display(", "))]
    GameVersionStrict(Vec<String>),

    /// Selects files compatible with the versions specified and related versions that are
    /// considered to not have breaking changes (determined using Modrinth's game version tag list)
    #[display("Game Version (Minor): {}", _0.iter().display(", "))]
    GameVersionMinor(Vec<String>),

    /// Selects files matching the channel provided or more stable channels
    #[display("Release Channel: {_0}")]
    ReleaseChannel(ReleaseChannel),

    /// Selects the files matching the provided regex
    #[display("Filename: {_0}")]
    Filename(String),
}

pub trait ProfileParameters {
    /// Get the game versions present, if self has `GameVersionStrict` or `GameVersionMinor`
    fn game_versions(&self) -> Option<&Vec<String>>;
    /// Get the first mod loader present, if self has `ModLoaderPrefer` or `ModLoaderAny`
    fn mod_loader(&self) -> Option<&ModLoader>;
    /// Get the game versions present, if self has `GameVersionStrict` or `GameVersionMinor`
    fn game_versions_mut(&mut self) -> Option<&mut Vec<String>>;
    /// Get the mod loaders present, if self has `ModLoaderPrefer` or `ModLoaderAny`
    fn mod_loaders_mut(&mut self) -> Option<&mut Vec<ModLoader>>;
}

impl ProfileParameters for Vec<Filter> {
    fn game_versions(&self) -> Option<&Vec<String>> {
        self.iter().find_map(|filter| match filter {
            Filter::GameVersionStrict(v) => Some(v),
            Filter::GameVersionMinor(v) => Some(v),
            _ => None,
        })
    }

    fn mod_loader(&self) -> Option<&ModLoader> {
        self.iter().find_map(|filter| match filter {
            Filter::ModLoaderPrefer(v) => Some(&v[0]),
            Filter::ModLoaderAny(v) => Some(&v[0]),
            _ => None,
        })
    }

    fn game_versions_mut(&mut self) -> Option<&mut Vec<String>> {
        self.iter_mut().find_map(|filter| match filter {
            Filter::GameVersionStrict(v) => Some(v),
            Filter::GameVersionMinor(v) => Some(v),
            _ => None,
        })
    }

    fn mod_loaders_mut(&mut self) -> Option<&mut Vec<ModLoader>> {
        self.iter_mut().find_map(|filter| match filter {
            Filter::ModLoaderPrefer(v) => Some(v),
            Filter::ModLoaderAny(v) => Some(v),
            _ => None,
        })
    }
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        discriminant(self) == discriminant(other)
    }
}

#[derive(Deserialize, Serialize, Debug, Display, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ReleaseChannel {
    Release,
    Beta,
    Alpha,
}
