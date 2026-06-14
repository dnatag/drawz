use serde::Serialize;

/// The structured response from every render call.
#[derive(Debug, Serialize)]
pub struct RenderResult {
    /// The rendered diagram text, or `None` if rendering failed.
    pub output: Option<String>,
    /// Whether the diagram fit within the width without degradation.
    pub fit: bool,
    /// Fatal errors that prevented rendering.
    pub errors: Vec<String>,
    /// Non-fatal warnings (truncation, suggestions).
    pub warnings: Vec<String>,
}

/// Mutable context threaded through rendering.
pub struct RenderContext {
    /// Available width for content (excluding frame borders if framed).
    pub inner_width: usize,
    /// Total output width.
    pub total_width: u16,
    /// Accumulated warnings.
    pub warnings: Vec<String>,
}
