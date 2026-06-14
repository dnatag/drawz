use serde::Serialize;

/// The structured response from every render call.
#[derive(Debug, Serialize)]
pub struct RenderResult {
    pub output: Option<String>,
    pub fit: bool,
    pub errors: Vec<String>,
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
