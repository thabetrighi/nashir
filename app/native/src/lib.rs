//! Nashir's native module, imported by the TSX interface as `nashir/native`
//! (`nsh.json` `"rust": "native"`).
//!
//! It exports three modules of functions:
//!
//! - [`Library`]: opening documents (in a background thread), their
//!   sections, outline, search, highlighting, change detection and images;
//! - [`Diagrams`]: Mermaid layout and SVG export;
//! - [`Desktop`]: file dialogs, launch arguments (file association), and
//!   revealing or opening files with the system.
//!
//! Data crosses into TSX as strings of JSON where it is structured: the
//! shapes are documented in `app/ui/native.ts`, which parses them.

mod desktop;
mod diagrams;
mod library;
mod util;

use nsh_core::prelude::*;

pub use crate::{desktop::Desktop, diagrams::Diagrams, library::Library};

/// The crate's UI set: what `App.tsx` imports from `nashir/native`.
#[native_module("nashir/native")]
fn native(registry: &mut Registry) {
    registry
        .module::<Library>()
        .module::<Diagrams>()
        .module::<Desktop>();
}
