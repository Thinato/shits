use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::{App, Mode};

impl App {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            _ => match self.mode {
                Mode::Normal => self.handle_normal_mode(key),
            },
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => self.move_cursor(-1, 0),
            KeyCode::Right | KeyCode::Char('l') => self.move_cursor(1, 0),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(0, -1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(0, 1),
            _ => {}
        }
    }

    fn move_cursor(&mut self, delta_col: i32, delta_row: i32) {
        self.cursor.row = apply_delta(self.cursor.row, delta_row);
        self.cursor.col = apply_delta(self.cursor.col, delta_col);
        self.ensure_cursor_visible();
    }

    fn ensure_cursor_visible(&mut self) {
        if self.cursor.row < self.viewport.row {
            self.viewport.row = self.cursor.row;
        } else if self.visible_rows > 0 {
            let bottom_edge = self.viewport.row + self.visible_rows.saturating_sub(1);
            if self.cursor.row > bottom_edge {
                self.viewport.row = self.cursor.row + 1 - self.visible_rows;
            }
        } else {
            self.viewport.row = self.cursor.row;
        }

        if self.cursor.col < self.viewport.col {
            self.viewport.col = self.cursor.col;
        } else if self.visible_cols > 0 {
            let right_edge = self.viewport.col + self.visible_cols.saturating_sub(1);
            if self.cursor.col > right_edge {
                self.viewport.col = self.cursor.col + 1 - self.visible_cols;
            }
        } else {
            self.viewport.col = self.cursor.col;
        }
    }
}

fn apply_delta(value: usize, delta: i32) -> usize {
    if delta < 0 {
        value.saturating_sub(delta.abs() as usize)
    } else {
        value.saturating_add(delta as usize)
    }
}
