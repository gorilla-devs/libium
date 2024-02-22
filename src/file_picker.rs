use crate::HOME;
use std::{
    io::Result,
    path::{Path, PathBuf},
};

#[cfg(feature = "gui")]
/// Use the system file picker to pick a file, with a `default` path (that is [not supported on linux](https://github.com/PolyMeilex/rfd/issues/42))
fn show_folder_picker(default: &Path, prompt: impl Into<String>) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_can_create_directories(true)
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

#[cfg(not(feature = "gui"))]
/// Use a terminal input to pick a file, with a `default` path
fn show_folder_picker(default: &Path, prompt: impl Into<String>) -> Option<PathBuf> {
    dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .default(default.display().to_string())
        .with_prompt(prompt)
        .report(false)
        .interact()
        .ok()
        .map(Into::into)
}

/// Pick a folder using the terminal or system file picker (depending on the features flag `gui`)
pub fn pick_folder(default: &Path, prompt: &str, name: &str) -> Result<Option<PathBuf>> {
    show_folder_picker(default, prompt)
        .map(|input| {
            let path = input
                .components()
                .map(|c| {
                    if c.as_os_str() == "~" {
                        HOME.as_os_str()
                    } else {
                        c.as_os_str()
                    }
                })
                .collect::<PathBuf>()
                .canonicalize()?;

            println!(
                "✔ \x1b[01m{}\x1b[0m · \x1b[32m{}\x1b[0m",
                name,
                path.display()
            );

            Ok(path)
        })
        .transpose()
}
