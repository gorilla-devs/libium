use crate::HOME;
use std::{
    io::Result,
    path::{Path, PathBuf},
};

#[cfg(feature = "gui")]
/// Use the system file picker to pick a file, with a `default` path that is [not supported on XDG](https://github.com/PolyMeilex/rfd/issues/42)
fn show_file_picker(default: &Path, prompt: &str) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_directory(default)
        .set_title(prompt)
        .pick_folder()
}

#[cfg(not(feature = "gui"))]
/// Use a terminal input to pick a file, with a `default` path
fn show_file_picker(default: &Path, prompt: &str) -> Option<PathBuf> {
    dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .default(default.display().to_string())
        .with_prompt(prompt)
        .report(false)
        .interact()
        .ok()
        .map(Into::into)
}

/// Pick a folder using  the terminal or file picker (depending on the features flags set)
///
/// For terminal output, display a `prompt`, and reply with the selected `name`
pub fn pick_folder(default: &Path, prompt: &str, name: &str) -> Result<Option<PathBuf>> {
    let input = show_file_picker(default, prompt);
    Ok(match input {
        Some(input) => {
            let mut path = PathBuf::new();
            let components = input.components();
            for c in components {
                path.push(if c.as_os_str() == "~" {
                    HOME.as_os_str()
                } else {
                    c.as_os_str()
                });
            }
            path = path.canonicalize()?;
            println!(
                "✔ \x1b[01m{}\x1b[0m · \x1b[32m{}\x1b[0m",
                name,
                path.display()
            );
            Some(path)
        },
        None => None,
    })
}
