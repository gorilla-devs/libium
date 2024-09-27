pub mod filters;
pub mod structs;

use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{Result, Seek, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub static DEFAULT_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    crate::HOME
        .join(".config")
        .join("ferium")
        .join("config.json")
});

fn open_config_file(path: &Path) -> Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)
}

/// Open the config file at `path`.
/// If it doesn't exist, a config file with an empty config will be created and opened.
pub fn get_file(path: &Path) -> Result<File> {
    if path.exists() {
        open_config_file(path)
    } else {
        create_dir_all(path.parent().unwrap())?;
        let mut file = open_config_file(path)?;
        write_file(&mut file, &structs::Config::default())?;
        Ok(file)
    }
}

/// Deserialise the given `input` into a config struct
pub fn deserialise(input: &str) -> serde_json::error::Result<structs::Config> {
    serde_json::from_str(input)
}

/// Serialise `config` and write it to `config_file`
pub fn write_file(config_file: &mut File, config: &structs::Config) -> Result<()> {
    let serialised = serde_json::to_string_pretty(config)?;
    config_file.set_len(0)?; // Clear the file contents
    config_file.rewind()?; // Set the cursor to the beginning
    config_file.write_all(serialised.as_bytes())?;
    config_file.rewind()?; // So that subsequent reads work properly
    Ok(())
}
