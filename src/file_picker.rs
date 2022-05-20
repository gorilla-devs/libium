use std::path::{Path, PathBuf};

// macOS, it can only use a synchronous file picker, and any GUI feature
#[cfg(all(target_os = "macos", any(feature = "gtk", feature = "xdg")))]
#[allow(clippy::unused_async)]
/// Use the file picker to pick a file, with a `default` path ([XDG not supported](https://github.com/PolyMeilex/rfd/issues/42))
pub async fn pick_folder(default: &Path, prompt: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

// Not macOS, other OSs can use the async version, and any GUI feature
#[cfg(all(not(target_os = "macos"), any(feature = "gtk", feature = "xdg")))]
/// Use the file picker to pick a file, with a `default` path ([XDG not supported](https://github.com/PolyMeilex/rfd/issues/42))
pub async fn pick_folder(default: &Path, prompt: &str) -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
        .await
        .map(|handle| handle.path().into())
}

// No GUI features
#[cfg(not(any(feature = "gtk", feature = "xdg")))]
#[allow(clippy::unused_async)]
pub async fn pick_folder(default: &Path, prompt: &str) -> Option<PathBuf> {
    use dialoguer::{theme::ColorfulTheme, Input};
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default.display().to_string())
        .interact()
        .ok()
        .map(Into::into)
}
