//! Rendering guarantee layer for AI agent terminal output.
//!
//! Converts structured JSON diagram descriptions into perfectly-aligned
//! ASCII/Unicode text for terminal display.

#![deny(missing_docs)]
#![allow(clippy::module_name_repetitions)]

/// Box-drawing frame wrapper.
pub mod frame;
/// Unicode-aware measurement utilities.
pub mod measure;
/// Mermaid subset parser.
pub mod mermaid;
/// Top-level render dispatch.
pub mod render;
/// Per-type diagram renderers.
pub mod renderers;
/// Render result and context types.
pub mod result;
/// Diagram input schema types.
pub mod schema;

pub use render::render;
pub use result::RenderResult;
pub use schema::{Diagram, DiagramInput};
