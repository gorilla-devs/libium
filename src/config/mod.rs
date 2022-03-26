pub mod structs;

use std::path::PathBuf;
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{AsyncSeekExt, AsyncWriteExt, Result},
};

/// Get the config file's path
pub fn config_file_path() -> PathBuf {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
}

/// Get the config file. If it doesn't exist, an empty config will be created and returned
pub async fn get_config_file(config_file_path: PathBuf) -> Result<File> {
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
            write_config(
                &mut file,
                &structs::Config {
                    active_profile: 0,
                    profiles: Vec::new(),
                },
            )
            .await?;

            Ok(file)
        }
        // If not just open and return the config file
        true => {
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(false)
                .open(config_file_path)
                .await
        }
    }
}

/// Serialise `config` and write it to `config_file`
pub async fn write_config(config_file: &mut File, config: &structs::Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?; // Serialise the config
    config_file.set_len(0).await?; // Truncate file to 0
    config_file.rewind().await?; // Rewind the file
    config_file.write_all(serialised.as_bytes()).await // Write the config to the config file
}
