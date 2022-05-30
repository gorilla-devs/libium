use crate::config::structs::ModIdentifier;
use ferinth::Ferinth;
use furse::Furse;
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use std::{fs, path::Path, sync::Arc};

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{}", .0)]
    IoError(#[from] std::io::Error),
    #[error("The mod does not exist")]
    NotFound,
    #[error("Couldn't find mod on modrinth or curseforge")]
    DoesNotExist,
    #[error("{}", .0)]
    ModrinthError(ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(furse::Error),
}
impl From<furse::Error> for Error {
    fn from(err: furse::Error) -> Self {
        if let furse::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::NotFound
            } else {
                Self::CurseForgeError(err)
            }
        } else {
            Self::CurseForgeError(err)
        }
    }
}
impl From<ferinth::Error> for Error {
    fn from(err: ferinth::Error) -> Self {
        if let ferinth::Error::ReqwestError(source) = &err {
            if Some(StatusCode::NOT_FOUND) == source.status() {
                Self::NotFound
            } else {
                Self::ModrinthError(err)
            }
        } else {
            Self::ModrinthError(err)
        }
    }
}

pub async fn scan<P>(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    mod_path: P,
) -> Result<Vec<ModIdentifier>>
where
    P: AsRef<Path>,
{
    let mut found_mods: Vec<ModIdentifier> = vec![];
    match get_modrinth_mod_by_hash(modrinth.clone(), &mod_path).await {
        Ok(mod_) => found_mods.push(mod_),
        Err(err) => {
            if !matches!(err, Error::NotFound) {
                return Err(err);
            }
        },
    }
    match get_curseforge_mod_by_hash(curseforge.clone(), &mod_path).await {
        Ok(mod_) => found_mods.push(mod_),
        Err(err) => {
            if !matches!(err, Error::NotFound) {
                return Err(err);
            }
        },
    }
    if found_mods.len() == 0 {
        return Err(Error::DoesNotExist);
    }
    Ok(found_mods)
}

async fn get_modrinth_mod_by_hash<P>(modrinth: Arc<Ferinth>, mod_path: P) -> Result<ModIdentifier>
where
    P: AsRef<Path>,
{
    let hash = Sha1::default().chain_update(fs::read(mod_path)?).finalize();
    let version = modrinth
        .get_version_from_file_hash(&format!("{:x}", hash))
        .await?;
    Ok(ModIdentifier::ModrinthProject(version.project_id))
}

async fn get_curseforge_mod_by_hash<P>(
    _curseforge: Arc<Furse>,
    _mod_path: P,
) -> Result<ModIdentifier>
where
    P: AsRef<Path>,
{
    // TODO 
    Err(Error::NotFound)
}
