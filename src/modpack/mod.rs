pub mod add;
pub mod curseforge;
pub mod modrinth;

use std::{
    io::{Read, Seek},
    path::Path,
};
use zip::{result::ZipResult, ZipArchive};

pub fn extract_modpack(input: impl Read + Seek, output_dir: &Path) -> ZipResult<()> {
    ZipArchive::new(input)?.extract(output_dir)
}
