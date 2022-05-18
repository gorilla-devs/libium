pub mod structs;

use serde_json::error::Result;
use std::io::{Read, Seek};
use structs::Manifest;
use zip::{result::ZipResult, ZipArchive};

/// Read the `input`'s manifest file to a string
pub fn read_manifest_file(input: impl Read + Seek) -> ZipResult<String> {
    let mut buffer = String::new();
    ZipArchive::new(input)?
        .by_name("manifest.json")?
        .read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Deserialise the given `input` into a manifest
pub fn deser_manifest(input: &str) -> Result<Manifest> {
    serde_json::from_str(input)
}
