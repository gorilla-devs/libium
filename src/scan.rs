use ferinth::Ferinth;
use furse::{cf_fingerprint, Furse};
use futures_util::TryFutureExt;
use sha1::{Digest, Sha1};
use std::{collections::HashMap, fs::read_dir, path::Path};
use tokio::{fs::read, try_join};

type Result<T> = std::result::Result<T, Error>;
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub enum Error {
    IOError(#[from] std::io::Error),
    ModrinthError(#[from] ferinth::Error),
    CurseForgeError(#[from] furse::Error),
}

/// Scan `dir_path` and return the filename, Modrinth project ID, and CurseForge mod ID for each JAR file
pub async fn scan(
    modrinth: &Ferinth,
    curseforge: &Furse,
    dir_path: impl AsRef<Path>,
    hashing_complete: impl Fn(),
) -> Result<Vec<(String, Option<String>, Option<i32>)>> {
    let mut filenames = HashMap::new();
    let mut mr_hashes = vec![];
    let mut cf_hashes = vec![];

    for entry in read_dir(dir_path)? {
        let path = entry?.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
        {
            let bytes = read(&path).await?;
            let cf_hash = cf_fingerprint(&bytes);

            if let Some(filename) = path.file_name() {
                filenames.insert(cf_hash, filename.to_owned());
            }
            mr_hashes.push(format!("{:x}", Sha1::digest(&bytes)));
            cf_hashes.push(cf_hash);
        }
    }

    hashing_complete();

    let (mr_results, cf_results) = try_join!(
        modrinth
            .get_versions_from_hashes(mr_hashes.clone())
            .map_err(Error::from),
        curseforge
            .get_fingerprint_matches(cf_hashes.clone())
            .map_err(Error::from),
    )?;

    // Elide explicit type parameters when https://github.com/rust-lang/rust/issues/90879 is resolved.
    let mut mr_results =
        HashMap::<_, _>::from_iter(mr_results.into_iter().map(|(k, v)| (k, v.project_id)));
    let mut cf_results = HashMap::<_, _>::from_iter(
        cf_results
            .exact_fingerprints
            .into_iter()
            .zip(cf_results.exact_matches.into_iter().map(|m| m.id)),
    );

    Ok(mr_hashes
        .into_iter()
        .zip(cf_hashes)
        .map(|(mr, cf)| {
            (
                filenames
                    .remove(&cf)
                    .expect("Missing filename in hashmap")
                    .to_string_lossy()
                    .into_owned(),
                mr_results.remove(&mr),
                cf_results.remove(&cf),
            )
        })
        .collect())
}
