use crate::HOME;
use std::{
    env::current_dir,
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
    use dialoguer::{theme::ColorfulTheme, Input};
    Input::with_theme(&ColorfulTheme::default())
        .default(default.display().to_string())
        .with_prompt(prompt)
        .report(false)
        .interact()
        .ok()
        .map(Into::into)
}

pub fn pick_folder(default: &Path, prompt: &str) -> Result<Option<PathBuf>> {
    let input = show_file_picker(default, prompt);
    Ok(match input {
        Some(input) => {
            let mut path = PathBuf::new();
            let components = input.components();
            for c in components {
                path = path.join(if c.as_os_str() == "~" {
                    HOME.to_owned()
                } else if c.as_os_str() == "." {
                    current_dir()?
                } else {
                    PathBuf::from(c.as_os_str())
                });
            }
            println!(
                "✔ \x1b[01mOutput Directory\x1b[0m · \x1b[32m{}\x1b[0m",
                path.display()
            );
            Some(path)
        },
        None => None,
    })
}
