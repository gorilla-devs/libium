pub mod add;
pub mod config;
pub mod file_picker;
pub mod misc;
pub mod modpack;
pub mod mutex_ext;
pub mod upgrade;
pub mod version_ext;

pub static HOME: once_cell::sync::Lazy<std::path::PathBuf> = once_cell::sync::Lazy::new(|| home::home_dir().expect("Could not get user's home directory"));

