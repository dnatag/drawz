//! Rendering guarantee layer for AI agent terminal output.
//!
//! Converts structured JSON diagram descriptions into perfectly-aligned
//! ASCII/Unicode text for terminal display.

#![deny(missing_docs)]
#![allow(clippy::module_name_repetitions)]

/// Diagram input schema types.
pub mod schema;
/// Render result and context types.
pub mod result;
/// Unicode-aware measurement utilities.
pub mod measure;
/// Box-drawing frame wrapper.
pub mod frame;
/// Top-level render dispatch.
pub mod render;
/// Per-type diagram renderers.
pub mod renderers;

pub use render::render;
pub use result::RenderResult;
pub use schema::{Diagram, DiagramInput};
