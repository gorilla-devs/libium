pub mod structs;

use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Result, Seek, Write},
    path::PathBuf,
};

/// Get the config file's path
pub fn config_file_path() -> PathBuf {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
}

/// Get the config file. If it doesn't exist, an empty config will be created and returned
pub fn get_config_file(config_file_path: PathBuf) -> Result<File> {
    match config_file_path.exists() {
        // If the file doesn't exist
        false => {
            // Create the config file directory
            create_dir_all(config_file_path.parent().unwrap())?;

            // Create and the open config file
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(true)
                .open(&config_file_path)?;

            // Write an empty config to the config file
            write_config(
                &file,
                &structs::Config {
                    active_profile: 0,
                    profiles: Vec::new(),
                },
            )?;

            Ok(file)
        }
        // If not just open and return the config file
        true => OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(false)
            .open(config_file_path),
    }
}

/// Serialise `config` and write it to `config_file`
pub fn write_config(mut config_file: &File, config: &structs::Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?; // Serialise the config
    config_file.set_len(0)?; // Truncate file to 0
    Seek::rewind(&mut config_file)?; // Rewind the file
    config_file.write_all(serialised.as_bytes()) // Write the config to the config file
}
