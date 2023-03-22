pub mod structs;

use async_zip::{error::Result, read::seek::ZipFileReader, StoredZipEntry};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};

/// Read the `input`'s manifest file
pub async fn read_manifest_file(
    input: impl AsyncRead + AsyncSeek + Unpin,
) -> Result<Option<String>> {
    let mut buffer = String::new();
    let zip_file = ZipFileReader::new(input).await?;
    if let Some(i) = zip_file
        .file()
        .entries()
        .iter()
        .map(StoredZipEntry::entry)
        .position(|entry| entry.filename() == "manifest.json")
    {
        zip_file
            .into_entry(i)
            .await?
            .read_to_string(&mut buffer)
            .await?;
        Ok(Some(buffer))
    } else {
        Ok(None)
    }
}
