pub mod add;
pub mod config;
pub mod file_picker;
pub mod misc;
pub mod modpack;
pub mod mutex_ext;
pub mod upgrade;
pub mod version_ext;
pub mod scan;

lazy_static::lazy_static! {
    pub static ref HOME: std::path::PathBuf = home::home_dir().expect("Could not get user's home directory");
}
