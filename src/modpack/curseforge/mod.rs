pub mod structs;

use async_zip::{error::Result, tokio::read::seek::ZipFileReader};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

/// Read the `input`'s manifest file
pub async fn read_manifest_file(
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
        .position(|&fname| fname == "manifest.json")
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
