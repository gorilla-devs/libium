pub mod structs;

use std::path::PathBuf;
use structs::Config;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, Result},
};

/// Get the default config file path
pub fn file_path() -> PathBuf {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
}

/// Open the config file at `path`.
/// If it doesn't exist, a config file with an empty config will be created and opened.
pub async fn get_file(path: PathBuf) -> Result<File> {
    if !path.exists() {
        // Create the config file directory
        create_dir_all(path.parent().unwrap()).await?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(path)
            .await?;
        write_file(
            &mut file,
            &Config {
                active_profile: 0,
                active_modpack: 0,
                profiles: Vec::new(),
                modpacks: Vec::new(),
            },
        )
        .await?;
        Ok(file)
    } else {
        OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(path)
            .await
    }
}

/// Read `config_file`.
/// Convenience function for dealing with the read buffer (or lack thereof).
pub async fn read_file(config_file: &mut File) -> Result<String> {
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer).await?;
    Ok(buffer)
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
