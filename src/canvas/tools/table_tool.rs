use crate::model::element::TableElement;
use egui::Pos2;
use uuid::Uuid;

pub struct TableTool {
    pub rows: usize,
    pub cols: usize,
    pub active_cell: Option<(Uuid, usize, usize)>, // (table_id, row, col)
    pub selected_text_range: Option<(Uuid, usize, usize, usize, usize)>, // (table_id, row, col, start, end)
    pub open_row_menu: Option<(Uuid, usize)>, // (table_id, row)
    pub dragging_row: Option<(Uuid, usize, f32)>, // (table_id, row, start_y)
}

impl Default for TableTool {
    fn default() -> Self {
        Self {
            rows: 4,
            cols: 3,
            active_cell: None,
            selected_text_range: None,
            open_row_menu: None,
            dragging_row: None,
        }
    }
}

impl TableTool {
    pub fn create_table(&self, pos: Pos2) -> TableElement {
        TableElement::new(Uuid::new_v4(), pos, self.rows, self.cols)
    }

    pub fn add_row(table: &mut TableElement) {
        let new_row = vec![String::new(); table.cols];
        table.cells.push(new_row);
        table.rows += 1;
        if table.row_colors.len() < table.rows {
            table.row_colors.resize(table.rows, None);
        }
    }

    pub fn remove_row(table: &mut TableElement, row_idx: usize) -> bool {
        if table.rows > 1 && row_idx < table.cells.len() {
            table.cells.remove(row_idx);
            table.rows -= 1;
            if row_idx < table.row_colors.len() {
                table.row_colors.remove(row_idx);
            }
            table.remove_row_spans(row_idx);
            true
        } else {
            false
        }
    }

    pub fn move_row_up(table: &mut TableElement, row: usize) {
        if row > 0 && row < table.rows {
            table.cells.swap(row, row - 1);
            if table.row_colors.len() < table.rows {
                table.row_colors.resize(table.rows, None);
            }
            table.row_colors.swap(row, row - 1);
            table.swap_row_spans(row, row - 1);
        }
    }

    pub fn move_row_down(table: &mut TableElement, row: usize) {
        if row + 1 < table.rows {
            table.cells.swap(row, row + 1);
            if table.row_colors.len() < table.rows {
                table.row_colors.resize(table.rows, None);
            }
            table.row_colors.swap(row, row + 1);
            table.swap_row_spans(row, row + 1);
        }
    }

    pub fn reorder_row(table: &mut TableElement, from_row: usize, to_row: usize) {
        if from_row == to_row || from_row >= table.rows || to_row >= table.rows {
            return;
        }
        let row = table.cells.remove(from_row);
        table.cells.insert(to_row, row);

        if table.row_colors.len() < table.rows {
            table.row_colors.resize(table.rows, None);
        }
        let c = table.row_colors.remove(from_row);
        table.row_colors.insert(to_row, c);

        table.reorder_row_spans(from_row, to_row);
    }

    pub fn add_column(table: &mut TableElement) {
        table.cols += 1;
        table.col_widths.push(110.0);
        for (r_idx, row) in table.cells.iter_mut().enumerate() {
            if r_idx == 0 {
                row.push(format!("Header {}", table.cols));
            } else {
                row.push(String::new());
            }
        }
    }

    pub fn remove_column(table: &mut TableElement, col_idx: usize) {
        if table.cols > 1 && col_idx < table.cols {
            table.cols -= 1;
            table.col_widths.remove(col_idx);
            for row in &mut table.cells {
                if col_idx < row.len() {
                    row.remove(col_idx);
                }
            }
        }
    }

    pub fn resize_columns(table: &mut TableElement, delta: f32) {
        for w in &mut table.col_widths {
            *w = (*w + delta).clamp(40.0, 400.0);
        }
    }
}
