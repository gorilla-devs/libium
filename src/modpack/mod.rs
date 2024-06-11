pub mod add;
pub mod curseforge;
pub mod modrinth;

use crate::read_wrapper;
use async_recursion::async_recursion;
use async_zip::{
    error::Result,
    tokio::{read::seek::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};
use std::{fs::read_dir, path::Path};
use tokio::{
    fs::{canonicalize, create_dir_all, metadata, read, File},
    io::{copy, AsyncBufRead, AsyncSeek, AsyncWrite},
};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

/// Extract the `input` zip file to `output_dir`
pub async fn extract_zip(
    input: impl AsyncBufRead + AsyncSeek + Unpin,
    output_dir: &Path,
) -> Result<()> {
    let mut zip = ZipFileReader::new(input.compat()).await?;
    for i in 0..zip.file().entries().len() {
        let entry = &zip.file().entries()[i];
        let path = output_dir.join(entry.filename().as_str()?);

        if entry.dir()? {
            create_dir_all(&path).await?;
        } else {
            if let Some(up_dir) = path.parent() {
                if !up_dir.exists() {
                    create_dir_all(up_dir).await?;
                }
            }
            copy(
                &mut zip.reader_without_entry(i).await?.compat(),
                &mut File::create(&path).await?,
            )
            .await?;
        }
    }
    Ok(())
}

/// Compress the input `dir`ectory (starting with `source`) to the given `writer`
///
/// Uses recursion to resolve directories.
/// Resolves symlinks as well.
#[async_recursion]
pub async fn compress_dir(
    writer: &mut ZipFileWriter<impl AsyncWrite + AsyncSeek + Unpin + Send>,
    source: impl AsRef<Path> + Send + 'async_recursion,
    dir: impl AsRef<Path> + Send + 'async_recursion,
    compression: Compression,
) -> Result<()> {
    for entry in read_dir(source.as_ref().join(dir.as_ref()))? {
        let entry = canonicalize(entry?.path()).await?;
        let meta = metadata(&entry).await?;
        if meta.is_dir() {
            compress_dir(
                writer,
                source.as_ref(),
                &dir.as_ref().join(entry.file_name().unwrap()),
                compression,
            )
            .await?;
        } else if meta.is_file() {
            let mut entry_builder = ZipEntryBuilder::new(
                dir.as_ref()
                    .join(entry.file_name().unwrap())
                    .to_string_lossy()
                    .as_ref()
                    .into(),
                compression,
            );
            #[cfg(unix)]
            {
                entry_builder = entry_builder.unix_permissions(
                    std::os::unix::fs::MetadataExt::mode(&meta)
                        .try_into()
                        .unwrap(),
                );
            }
            writer
                .write_entry_whole(entry_builder, &read(entry).await?)
                .await?;
        }
    }
    Ok(())
}

/// Returns the contents of the `file_name` from the provided `input` zip file if it exists
pub async fn read_file_from_zip(
    input: impl AsyncBufRead + AsyncSeek + Unpin,
    file_name: &str,
) -> Result<Option<String>> {
    let zip_file = ZipFileReader::new(input.compat()).await?;
    if let Some(i) = zip_file
        .file()
        .entries()
        .iter()
        .position(|entry| entry.filename().as_str().is_ok_and(|f| f == file_name))
    {
        Ok(Some(
            read_wrapper(zip_file.into_entry(i).await?.compat()).await?,
        ))
    } else {
        Ok(None)
    }
}
