use super::element::BoardElement;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct UndoStack {
    undo_frames: VecDeque<Vec<BoardElement>>,
    redo_frames: VecDeque<Vec<BoardElement>>,
    max_depth: usize,
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new(100)
    }
}

impl UndoStack {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_frames: VecDeque::with_capacity(max_depth + 1),
            redo_frames: VecDeque::new(),
            max_depth,
        }
    }

    /// Pushes a checkpoint of the board's elements before a mutation occurs.
    pub fn push_checkpoint(&mut self, current_elements: Vec<BoardElement>) {
        if self.undo_frames.len() >= self.max_depth {
            self.undo_frames.pop_front();
        }
        self.undo_frames.push_back(current_elements);
        self.redo_frames.clear();
    }

    /// Undoes the last mutation, returning the previous elements state while saving current into redo
    pub fn undo(&mut self, current_elements: Vec<BoardElement>) -> Option<Vec<BoardElement>> {
        let prev = self.undo_frames.pop_back()?;
        self.redo_frames.push_back(current_elements);
        Some(prev)
    }

    /// Redoes the last undone mutation, returning the future state while saving current into undo
    pub fn redo(&mut self, current_elements: Vec<BoardElement>) -> Option<Vec<BoardElement>> {
        let next = self.redo_frames.pop_back()?;
        self.undo_frames.push_back(current_elements);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_frames.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_frames.clear();
        self.redo_frames.clear();
    }

    pub fn depth(&self) -> usize {
        self.undo_frames.len()
    }
}
