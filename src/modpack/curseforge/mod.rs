pub mod structs;

use async_zip::error::Result;
use tokio::io::{AsyncRead, AsyncSeek};

use super::read_from_zip;

/// Read the `input`'s manifest file
pub async fn read_manifest_file(
    input: impl AsyncRead + AsyncSeek + Unpin,
) -> Result<Option<String>> {
    read_from_zip(input, "manifest.json").await
}
