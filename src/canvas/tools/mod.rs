pub mod eraser;
pub mod pen;
pub mod select;
pub mod shapes;
pub mod table_tool;
pub mod text_tool;
pub mod wire;

pub use eraser::EraserTool;
pub use pen::PenTool;
pub use select::{DragMode, ResizeHandle, SelectTool};
pub use shapes::ShapeTool;
pub use table_tool::TableTool;
pub use text_tool::TextTool;
pub use wire::WireTool;

use crate::model::electronics::ComponentKind;
use crate::model::element::ShapeKind;

#[derive(Clone, Debug, PartialEq)]
pub enum ActiveTool {
    Select,
    VertexEdit,
    Pan,
    Pen,
    Highlighter,
    Eraser,
    Shape(ShapeKind),
    Wire,
    Text,
    StickyNote,
    Table,
    Component(ComponentKind),
}

impl Default for ActiveTool {
    fn default() -> Self {
        ActiveTool::Select
    }
}

impl ActiveTool {
    pub fn display_name(&self) -> &'static str {
        match self {
            ActiveTool::Select => "Select (V)",
            ActiveTool::VertexEdit => "Vertex Edit (A)",
            ActiveTool::Pan => "Pan Canvas (Hand)",
            ActiveTool::Pen => "Pen (P)",
            ActiveTool::Highlighter => "Highlighter (H)",
            ActiveTool::Eraser => "Eraser (E)",
            ActiveTool::Shape(kind) => kind.display_name(),
            ActiveTool::Wire => "Magnetic Wire (W)",
            ActiveTool::Text => "Text (T)",
            ActiveTool::StickyNote => "Sticky Note (N)",
            ActiveTool::Table => "Table",
            ActiveTool::Component(comp) => comp.display_name(),
        }
    }
}
