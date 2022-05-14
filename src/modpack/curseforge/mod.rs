pub mod structs;

use serde_json::error::Result;
use std::io::{Read, Seek};
use structs::Manifest;
use zip::{result::ZipResult, ZipArchive};

pub fn read_manifest_file(file: impl Read + Seek) -> ZipResult<String> {
    let mut buffer = String::new();
    ZipArchive::new(file)?
        .by_name("manifest.json")?
        .read_to_string(&mut buffer)?;
    Ok(buffer)
}

pub fn deser_manifest(input: &str) -> Result<Manifest> {
    serde_json::from_str(input)
}
