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

/// Get the config file. If it doesn't exist, an empty config will be created
pub async fn get_file(config_file_path: PathBuf) -> Result<File> {
    match config_file_path.exists() {
        // If the file doesn't exist
        false => {
            // Create the config file directory
            create_dir_all(config_file_path.parent().unwrap()).await?;

            // Create and the open config file
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(true)
                .open(&config_file_path)
                .await?;

            // Write an empty config to the config file
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
        },
        // If not just open and return the config file
        true => {
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(false)
                .open(&config_file_path)
                .await
        },
    }
}

/// Read the config file to a string
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
    config_file.set_len(0).await?; // Truncate file to 0
    config_file.rewind().await?; // Set the cursor to the beginning
    config_file.write_all(serialised.as_bytes()).await?;
    config_file.rewind().await?; // So that subsequent reads work properly
    Ok(())
}
