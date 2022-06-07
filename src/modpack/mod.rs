pub mod add;
pub mod curseforge;
pub mod modrinth;

use std::{
    fs::File,
    io::{copy, Read, Seek},
    path::Path,
};
use tokio::fs::create_dir_all;
use zip::{
    result::{ZipError, ZipResult},
    ZipArchive,
};

/// Extract the `input` zip file to `output_dir`
pub async fn extract_zip(input: impl Read + Seek, output_dir: &Path) -> ZipResult<()> {
    let mut zip = ZipArchive::new(input)?;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let filepath = file
            .enclosed_name()
            .ok_or(ZipError::InvalidArchive("Invalid file path"))?;

        let outpath = output_dir.join(filepath);

        if file.name().ends_with('/') {
            create_dir_all(&outpath).await?;
        } else {
            if let Some(up_dir) = outpath.parent() {
                if !up_dir.exists() {
                    create_dir_all(&up_dir).await?;
                }
            }
            copy(&mut file, &mut File::create(&outpath)?)?;
        }
    }
    Ok(())
}
