pub mod add;
pub mod curseforge;
pub mod modrinth;

use async_recursion::async_recursion;
use async_zip::{
    error::Result,
    tokio::{read::seek::ZipFileReader, write::ZipFileWriter},
    Compression, ZipEntryBuilder,
};
use std::{fs::read_dir, path::Path};
use tokio::{
    fs::{canonicalize, create_dir_all, metadata, read, File},
    io::{copy, AsyncRead, AsyncReadExt, AsyncSeek, AsyncWrite},
};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

/// Extract the `input` zip file to `output_dir`
pub async fn extract_zip(
    input: impl AsyncRead + AsyncSeek + Unpin,
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
pub async fn compress_dir<W: AsyncWrite + AsyncSeek + Unpin + Send>(
    writer: &mut ZipFileWriter<W>,
    source: &Path,
    dir: &Path,
    compression: Compression,
) -> Result<()> {
    for entry in read_dir(source.join(dir))? {
        let entry = canonicalize(entry?.path()).await?;
        let meta = metadata(&entry).await?;
        if meta.is_dir() {
            compress_dir(
                writer,
                source,
                &dir.join(entry.file_name().unwrap()),
                compression,
            )
            .await?;
        } else if meta.is_file() {
            writer
                .write_entry_whole(
                    ZipEntryBuilder::new(
                        dir.join(entry.file_name().unwrap())
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
    Ok(())
}

pub async fn read_from_zip(
    input: impl AsyncRead + AsyncSeek + Unpin,
    file_name: &str,
) -> Result<Option<String>> {
    let mut buffer = String::new();
    let zip_file = ZipFileReader::new(input.compat()).await?;
    if let Some(i) = zip_file
        .file()
        .entries()
        .iter()
        .flat_map(|entry| entry.filename().as_str())
        .position(|fname| fname == file_name)
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
