pub mod structs;

use async_zip::{
    error::Result,
    tokio::{read::seek::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};
use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};
use tokio::{
    fs::{canonicalize, read, File},
    io::{AsyncRead, AsyncReadExt, AsyncSeek},
};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

/// Read the `input`'s metadata file to a string
pub async fn read_metadata_file(
    input: impl AsyncRead + AsyncSeek + Unpin,
) -> Result<Option<String>> {
    let mut buffer = String::new();
    let zip_file = ZipFileReader::new(input.compat()).await?;
    if let Some(i) = zip_file
        .file()
        .entries()
        .iter()
        .map(|entry| entry.filename().as_str())
        .collect::<Result<Vec<&str>>>()?
        .iter()
        .position(|&fname| fname == "modrinth.index.json")
    {
        zip_file
            .into_entry(i)
            .await?
            .compat()
            .read_to_string(&mut buffer)
            .await?;
        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}

/// Create a Modrinth modpack at `output` using the provided `metadata` and optional `overrides`
pub async fn create(
    output: &Path,
    metadata: &str,
    overrides: Option<&Path>,
    additional_mods: Option<&Path>,
) -> Result<File> {
    let compression = Compression::Deflate;
    let mut writer = ZipFileWriter::new(File::create(output).await?.compat());
    writer
        .write_entry_whole(
            ZipEntryBuilder::new("modrinth.index.json".into(), compression),
            metadata.as_bytes(),
        )
        .await?;
    if let Some(overrides) = overrides {
        super::compress_dir(
            &mut writer,
            overrides.parent().unwrap(),
            &PathBuf::from("overrides"),
            compression,
        )
        .await?;
    }
    if let Some(path) = additional_mods {
        for entry in read_dir(path)? {
            let entry = entry?;
            if entry.metadata()?.is_file() {
                let entry = canonicalize(entry.path()).await?;
                writer
                    .write_entry_whole(
                        ZipEntryBuilder::new(
                            PathBuf::from("overrides")
                                .join("mods")
                                .join(entry.file_name().unwrap())
                                .to_string_lossy()
                                .as_ref()
                                .into(),
                            compression,
                        ),
                        &read(entry).await?,
                    )
                    .await?;
            }
        }
    }
    writer.close().await.map(Compat::into_inner)
}
