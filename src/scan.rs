use ferinth::Ferinth;
use furse::{cf_fingerprint, Furse};
use sha1::{Digest, Sha1};
use std::{fs::read_dir, path::Path};
use tokio::fs::read;

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    IOError(#[from] std::io::Error),
    ModrinthError(#[from] ferinth::Error),
    CurseForgeError(#[from] furse::Error),
}

/// Scan the given `folder_path` and return Modrinth project IDs and CurseForge mod IDs
pub async fn scan(
    modrinth: &Ferinth,
    curseforge: &Furse,
    folder_path: impl AsRef<Path>,
) -> Result<(Vec<String>, Vec<i32>)> {
    let mut mr_hashes = vec![];
    let mut cf_hashes = vec![];

    for entry in read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        {
            let bytes = read(path).await?;
            mr_hashes.push(format!("{:x}", Sha1::digest(&bytes)));
            cf_hashes.push(cf_fingerprint(&bytes));
        }
    }

    Ok((
        modrinth
            .get_versions_from_hashes(mr_hashes)
            .await?
            .into_values()
            .map(|version| version.project_id)
            .collect(),
        curseforge
            .get_fingerprint_matches(cf_hashes)
            .await?
            .exact_matches
            .into_iter()
            .map(|m| m.id)
            .collect(),
    ))
}
