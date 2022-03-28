pub mod add;
pub mod config;
pub mod file_picker;
pub mod misc;

// Get the home directory when the program starts
lazy_static::lazy_static! {
    pub static ref HOME: std::path::PathBuf = home::home_dir().expect("Could not get user's home directory");
}
