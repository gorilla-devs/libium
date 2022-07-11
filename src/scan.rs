use ferinth::{structures::version_structs::Version, Ferinth};
use furse::{
    structures::{
        file_structs::{File, HashAlgo},
        fingerprint_structs::FingerprintMatches,
    },
    Furse,
};
use sha1::{Digest, Sha1};
use std::{
    collections::HashMap,
    fs,
    path::{PathBuf, Path},
    sync::Arc,
};

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{}", .0)]
    IOError(#[from] std::io::Error),
    #[error("{}", .0)]
    ModrinthError(#[from] ferinth::Error),
    #[error("{}", .0)]
    CurseForgeError(#[from] furse::Error),
}

/// Scan the given `mod_paths` and return a CurseForge file and Modrinth version for each path
pub async fn scan(
    modrinth: Arc<Ferinth>,
    curseforge: Arc<Furse>,
    mod_paths: Vec<PathBuf>,
) -> Result<HashMap<PathBuf, (Option<Version>, Option<File>)>> {
    let modrinth_mods = get_modrinth_mods_by_hash(modrinth.clone(), mod_paths.clone()).await?;
    let curseforge_mods =
        get_curseforge_mods_by_hash(curseforge.clone(), mod_paths.clone()).await?;

    let mut file_hashes: HashMap<String, &Path> = HashMap::new();
    for mod_path in &mod_paths {
        let hash = Sha1::default().chain_update(fs::read(mod_path)?).finalize();
        file_hashes.entry(format!("{:x}", hash)).or_insert(mod_path);
    }

    let mut mods: HashMap<PathBuf, (Option<Version>, Option<File>)> = mod_paths
        .iter()
        .map(|path| (path.to_path_buf(), (None, None)))
        .collect();

    for mod_ in modrinth_mods {
        if let Some(path) = file_hashes.get(&mod_.0) {
            mods.entry(path.to_path_buf()).or_default().0 = Some(mod_.1);
        }
    }
    for mod_ in curseforge_mods {
        if let Some(hash) = mod_
            .hashes
            .iter()
            .find(|hash| hash.algo == HashAlgo::Sha1)
        {
            if let Some(path) = file_hashes.get_mut(&hash.value) {
                mods.entry(path.to_path_buf()).or_default().1 = Some(mod_);
            }
        }
    }
    Ok(mods)
}

/// Search for the given `mod_paths` on Modrinth by hash and return a version for each hash
pub async fn get_modrinth_mods_by_hash(
    modrinth: Arc<Ferinth>,
    mod_paths: Vec<PathBuf>,
) -> Result<HashMap<String, Version>> {
    let mut hashes = vec![];
    for file in mod_paths {
        let hash = Sha1::default().chain_update(fs::read(file)?).finalize();
        hashes.push(format!("{:x}", hash));
    }
    let versions = modrinth.get_versions_from_hashes(hashes).await?;
    Ok(versions)
}

/// Search for the given `mod_paths` on CurseForge by hash
pub async fn get_curseforge_mods_by_hash(
    curseforge: Arc<Furse>,
    mod_paths: Vec<PathBuf>,
) -> Result<Vec<File>> {
    let mut file_hashes = vec![];
    for file in mod_paths {
        file_hashes.push(furse::cf_fingerprint(bytes::Bytes::from(fs::read(file)?)));
    }
    let FingerprintMatches { exact_matches, .. } =
        curseforge.get_fingerprint_matches(file_hashes).await?;
    let mut files = vec![];
    for r#match in exact_matches {
        files.push(r#match.file);
    }
    Ok(files)
}
