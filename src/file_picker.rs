use std::path::PathBuf;

// macOS can only use a sync file picker
#[cfg(all(target_os = "macos", feature = "gui"))]
#[allow(clippy::unused_async)]
/// Use the file picker to pick a file, with a `default` path
pub async fn pick_folder(default: &PathBuf, prompt: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

// Other OSs can use the async version
#[cfg(all(not(target_os = "macos"), feature = "gui"))]
/// Use the file picker to pick a file, with a `default` path
pub async fn pick_folder(default: &PathBuf, prompt: &str) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
        .await
        .map(|handle| handle.path().into())
}

#[cfg(not(feature = "gui"))]
#[allow(clippy::unused_async)]
pub async fn pick_folder(default: &PathBuf, prompt: &str) -> Option<PathBuf> {
    use dialoguer::{theme::ColorfulTheme, Input};
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default.display().to_string())
        .interact()
        .ok()
        .map(|string| string.into())
}
