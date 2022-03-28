use std::path::PathBuf;

// macOS can only use a sync file picker
#[cfg(all(target_os = "macos", feature = "gui"))]
#[allow(clippy::unused_async)]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &PathBuf) -> Option<PathBuf> {
    rfd::FileDialog::new().set_directory(path).pick_folder()
}

// Other OSs can use the async version
#[cfg(all(not(target_os = "macos"), feature = "gui"))]
/// Use the file picker to pick a file, defaulting to `path`
pub async fn pick_folder(path: &PathBuf) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(path)
        .pick_folder()
        .await
        .map(|handle| handle.path().into())
}

#[cfg(not(feature = "gui"))]
#[allow(clippy::unused_async)]
pub async fn pick_folder(_: &PathBuf) -> Option<PathBuf> {
    use dialoguer::{theme::ColorfulTheme, Input};

    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a mod output directory")
        .interact()
        .ok()?;
    Some(input.into())
}
