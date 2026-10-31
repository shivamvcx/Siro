use crate::model::element::TextElement;
use egui::{Color32, Pos2};
use uuid::Uuid;

pub struct TextTool {
    pub is_sticky: bool,
    pub sticky_color: Color32,
    pub editing_text_id: Option<Uuid>,
    pub pending_text_buffer: String,
}

impl Default for TextTool {
    fn default() -> Self {
        Self {
            is_sticky: false,
            sticky_color: Color32::from_rgb(255, 240, 130), // Classic yellow
            editing_text_id: None,
            pending_text_buffer: String::new(),
        }
    }
}

impl TextTool {
    pub fn new_text_tool() -> Self {
        Self {
            is_sticky: false,
            sticky_color: Color32::from_rgb(255, 240, 130),
            editing_text_id: None,
            pending_text_buffer: String::new(),
        }
    }

    pub fn new_sticky_tool(color: Color32) -> Self {
        Self {
            is_sticky: true,
            sticky_color: color,
            editing_text_id: None,
            pending_text_buffer: String::new(),
        }
    }

    pub fn create_element(&self, pos: Pos2, text: impl Into<String>) -> TextElement {
        let text_str = text.into();
        let id = Uuid::new_v4();
        if self.is_sticky {
            TextElement::new_sticky(id, pos, text_str, self.sticky_color)
        } else {
            TextElement::new_text(id, pos, text_str)
        }
    }
}
