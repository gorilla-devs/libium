pub mod structs;

use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, Result},
};

use self::structs::Config;

/// Default config location
pub static DEFAULT_CONFIG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
});

/// Get the default config file path
/// TODO: This fn call can be removed, cosnt value can be used directly
pub fn file_path() -> PathBuf {
    DEFAULT_CONFIG_PATH.clone()
}

#[inline]
pub async fn open_config_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)
        .await
}

pub async fn generate_default_config_file(path: &Path) -> Result<File> {
    // Create the config file directory
    create_dir_all(path.parent().unwrap()).await?;
    let mut file = open_config_file(path).await?;
    write_file(&mut file, &Config::default()).await?;
    Ok(file)
}

/// Open the config file at `path`.
/// If it doesn't exist, a config file with an empty config will be created and opened.
pub async fn get_file(path: PathBuf) -> Result<File> {
    if path.exists() {
        open_config_file(&path).await
    } else {
        generate_default_config_file(&path).await
    }
}

/// Read `config_file`.
/// Convenience function for dealing with the read buffer (or lack thereof).
pub async fn read_file(config_file: &mut File) -> Result<String> {
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer).await?;
    Ok(buffer)
}

/// Alternative for [read_file]
pub async fn read_from_file_path(file_path: &Path) -> Result<String> {
    tokio::fs::read_to_string(file_path).await
}

/// Deserialise the given `input` into a config struct
pub fn deserialise(input: &str) -> serde_json::error::Result<Config> {
    serde_json::from_str(input)
}

/// Serialise `config` and write it to `config_file`
pub async fn write_file(config_file: &mut File, config: &Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?;
    config_file.set_len(0).await?; // Clear the file contents
    config_file.rewind().await?; // Set the cursor to the beginning
    config_file.write_all(serialised.as_bytes()).await?;
    config_file.rewind().await?; // So that subsequent reads work properly
    Ok(())
}
