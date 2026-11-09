//! `Diagrams`: Mermaid layout and export.

use nashir_engine::mermaid;
use nsh_core::{
    file_dialog::{FileFilter, SaveDialog},
    prelude::*,
};

/// Mermaid diagrams, laid out natively.
#[module]
pub struct Diagrams;

#[module]
impl Diagrams {
    /// Lays out the Mermaid `source`. Returns JSON: `{"ok":true,"scene":{…}}`
    /// or `{"ok":false,"error":"unsupported","detail":"quadrantChart"}`
    /// (`MermaidResult` in `ui/native.ts`).
    pub fn layout(source: String) -> String {
        mermaid::render_json(&source)
    }

    /// The diagram keywords this engine draws.
    pub fn supported() -> Vec<String> {
        mermaid::SUPPORTED.iter().map(|s| s.to_string()).collect()
    }

    /// Asks where to save the diagram as SVG (suggesting `name`), then
    /// writes it in the light or dark palette. Resolves `true` when saved,
    /// `false` when the user cancelled; rejects when it cannot be written.
    pub async fn export_svg(source: String, dark: bool, name: String) -> Result<bool, String> {
        let scene = mermaid::render(&source).map_err(|e| e.code().to_owned())?;
        let svg = mermaid::svg::to_svg(&scene, dark);
        let picked = pick_save_path(
            SaveDialog::new()
                .file_name(if name.ends_with(".svg") {
                    name
                } else {
                    format!("{name}.svg")
                })
                .filter(FileFilter::new("SVG", ["svg"])),
        )
        .await?;
        match picked {
            Some(path) => {
                std::fs::write(&path, svg).map_err(|e| e.to_string())?;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}
