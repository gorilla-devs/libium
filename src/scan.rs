use ferinth::{structures::version_structs::Version, Ferinth};
use furse::{structures::file_structs::File, Furse};
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModPlatform {
    Modrinth,
    Curseforge,
}

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
    #[error("Could not find mod on Modrinth or CurseForge")]
    DoesNotExist,
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
}

pub async fn scan(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    mod_path: &Path,
) -> Result<(Option<String>, Option<i32>)> {
    Ok((
        get_modrinth_mod_by_hash(modrinth.clone(), mod_path)
            .await?
            .map(|version| version.project_id),
        get_curseforge_mod_by_hash(curseforge.clone(), mod_path)
            .await?
            .map(|file| file.mod_id),
    ))
}

/// Get the version of the mod at `mod_path`
pub async fn get_modrinth_mod_by_hash(
    modrinth: Arc<Ferinth>,
    mod_path: &Path,
) -> Result<Option<Version>> {
    let hash = Sha1::default().chain_update(fs::read(mod_path)?).finalize();
    let result = modrinth
        .get_version_from_file_hash(&format!("{:x}", hash))
        .await;
    match result {
        Ok(version) => Ok(Some(version)),
        Err(err) => {
            if let ferinth::Error::ReqwestError(source) = &err {
                if Some(StatusCode::NOT_FOUND) == source.status() {
                    Ok(None)
                } else {
                    Err(err.into())
                }
            } else {
                Err(err.into())
            }
        },
    }
}

/// Get the file of the mod at `mod_path`
pub async fn get_curseforge_mod_by_hash(
    curseforge: Arc<Furse>,
    mod_path: &Path,
) -> Result<Option<File>> {
    let bytes = fs::read(mod_path)?;
    let mut matches = curseforge
        .get_fingerprint_matches(vec![bytes.into()])
        .await?
        .exact_matches;
    if matches.is_empty() {
        Ok(None)
    } else {
        Ok(Some(matches.swap_remove(0).file))
    }
}
