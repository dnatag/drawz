use crate::schema::Diagram;

/// Render a diagram to a framed ASCII/Unicode string within the given width.
pub fn render(diagram: &Diagram, _width: u16) -> String {
    match diagram {
        Diagram::Flow(d) => format!("[ flow: {} ]", d.steps.as_ref().map_or(
            d.nodes.as_ref().map_or(0, |n| n.len()),
            |s| s.len()
        )),
        Diagram::State(d) => format!("[ state: {} transitions ]", d.transitions.len()),
        Diagram::Tree(_) => "[ tree ]".to_string(),
        Diagram::Sequence(d) => format!("[ sequence: {} actors ]", d.actors.len()),
        Diagram::Table(d) => format!("[ table: {}x{} ]", d.headers.len(), d.rows.len()),
        Diagram::Dag(d) => format!("[ dag: {} edges ]", d.edges.len()),
        Diagram::Freeform(_) => "[ freeform ]".to_string(),
        Diagram::Mermaid(d) => format!("[ mermaid: {} chars ]", d.code.len()),
    }
}
