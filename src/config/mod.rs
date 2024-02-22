pub mod structs;

use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt, Result},
};

pub static DEFAULT_CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
});

async fn open_config_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)
        .await
}

/// Open the config file at `path`.
/// If it doesn't exist, a config file with an empty config will be created and opened.
pub async fn get_file(path: &Path) -> Result<File> {
    if path.exists() {
        open_config_file(path).await
    } else {
        create_dir_all(path.parent().unwrap()).await?;
        let mut file = open_config_file(path).await?;
        write_file(&mut file, &structs::Config::default()).await?;
        Ok(file)
    }
}

/// Deserialise the given `input` into a config struct
pub fn deserialise(input: &str) -> serde_json::error::Result<structs::Config> {
    serde_json::from_str(input)
}

/// Serialise `config` and write it to `config_file`
pub async fn write_file(config_file: &mut File, config: &structs::Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?;
    config_file.set_len(0).await?; // Clear the file contents
    config_file.rewind().await?; // Set the cursor to the beginning
    config_file.write_all(serialised.as_bytes()).await?;
    config_file.rewind().await?; // So that subsequent reads work properly
    Ok(())
}
