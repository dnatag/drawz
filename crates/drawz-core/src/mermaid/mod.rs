//! Mermaid subset parser — converts flowchart, sequenceDiagram, stateDiagram
//! to internal diagram types.

mod flowchart;
mod helpers;
mod parse;
mod sequence;
mod state;

pub use parse::parse;
