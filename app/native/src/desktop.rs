//! `Desktop`: the operating system's file dialogs, launch arguments and
//! file opening.

use nashir_engine::{is_markdown_path, MARKDOWN_EXTENSIONS};
use nsh_core::{
    file_dialog::{FileFilter, OpenDialog},
    prelude::*,
};

use crate::util::path_from_uri;

/// Desktop services.
#[module]
pub struct Desktop;

#[module]
impl Desktop {
    /// Shows the system's Open dialog for Markdown files (several may be
    /// chosen). Resolves with the chosen paths, or none when cancelled.
    pub async fn pick_markdown(title: String, filter_label: String) -> Result<Vec<String>, String> {
        let picked = pick_paths(
            OpenDialog::files()
                .multiple(true)
                .filter(FileFilter::new(filter_label, MARKDOWN_EXTENSIONS.iter().copied()))
                .title(title),
        )
        .await?;
        Ok(picked
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect())
    }

    /// The Markdown files Nashir was started with: what Explorer passes when
    /// a `.md` file is double-clicked (file association) or dropped on the
    /// app's icon.
    pub fn launch_paths() -> Vec<String> {
        std::env::args()
            .skip(1)
            .filter(|a| !a.starts_with('-'))
            .map(|a| path_from_uri(&a))
            .filter(|p| p.is_file())
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    }

    /// `uris` (`file:` URLs from a file drop) as paths, Markdown files only.
    pub fn markdown_paths(uris: Vec<String>) -> Vec<String> {
        uris.into_iter()
            .map(|u| path_from_uri(&u).to_string_lossy().into_owned())
            .filter(|p| is_markdown_path(p))
            .collect()
    }

    /// Shows the file selected in File Explorer (Finder, the file manager).
    pub async fn reveal(path: String) -> Result<(), String> {
        reveal_path(path_from_uri(&path)).await
    }

    /// Opens the file with the application the system associates with it.
    pub async fn open_with_system(path: String) -> Result<(), String> {
        open_path(path_from_uri(&path)).await
    }
}
