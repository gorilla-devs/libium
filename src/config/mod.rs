pub mod structs;

use std::path::PathBuf;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, Result},
};

/// Get the config file's path
pub fn file_path() -> PathBuf {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
}

/// Get the config file. If it doesn't exist, an empty config will be created and returned
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
                &structs::Config {
                    active_profile: 0,
                    profiles: Vec::new(),
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
                .open(config_file_path)
                .await
        },
    }
}

/// Read the config file to string
pub async fn read_file(config_file: &mut File) -> Result<String> {
    let mut buffer = String::new();
    config_file.read_to_string(&mut buffer).await?;
    Ok(buffer)
}

/// Deserialise the given `input` into a config struct
pub fn deserialise(input: &str) -> serde_json::error::Result<structs::Config> {
    serde_json::from_str(input)
}

/// Serialise `config` and write it to `config_file`
pub async fn write_file(config_file: &mut File, config: &structs::Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?; // Serialise the config
    config_file.set_len(0).await?; // Truncate file to 0
    config_file.rewind().await?; // Rewind the file
    config_file.write_all(serialised.as_bytes()).await // Write the config to the config file
}
