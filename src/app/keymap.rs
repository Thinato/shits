use std::{
    cmp::Ordering,
    fs::File,
    io::{self, Write},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Cursor;

use super::{App, CellId, InsertState, Mode};

impl App {
    pub(crate) fn on_key_event(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') => {
                    self.handle_save_command(self.file_name.clone());
                    return;
                }
                KeyCode::Char('q') => {
                    self.quit();
                    return;
                }
                KeyCode::Char('c') => {
                    self.quit();
                    return;
                }
                _ => {}
            }
        }

        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert(_) => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('b') => {
                self.clear_command_buffer();
                self.move_cursor(-5, 0);
            }
            KeyCode::Char('w') => {
                self.clear_command_buffer();
                self.move_cursor(5, 0);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.clear_command_buffer();
                self.move_cursor(-1, 0);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.clear_command_buffer();
                self.move_cursor(1, 0);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.clear_command_buffer();
                self.move_cursor(0, -1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.clear_command_buffer();
                self.move_cursor(0, 1);
            }
            KeyCode::Char('g') if key.modifiers.is_empty() => match self.command_buffer.as_str() {
                "g" => {
                    self.go_to_first_row();
                    self.clear_command_buffer();
                }
                _ => {
                    self.command_buffer.clear();
                    self.command_buffer.push('g');
                }
            },
            KeyCode::Char('G') => {
                self.command_buffer.clear();
                self.go_to_last_row_with_value();
            }
            KeyCode::Char('o') if key.modifiers.is_empty() => {
                self.clear_command_buffer();
                self.insert_row_below_and_edit();
            }
            KeyCode::Char('O') => {
                self.clear_command_buffer();
                self.insert_row_above_and_edit();
            }
            KeyCode::Char('d') if key.modifiers.is_empty() => match self.command_buffer.as_str() {
                "d" => {
                    self.delete_current_row();
                    self.clear_command_buffer();
                }
                _ => {
                    self.command_buffer.clear();
                    self.command_buffer.push('d');
                }
            },
            KeyCode::Char('y') if key.modifiers.is_empty() => match self.command_buffer.as_str() {
                "y" => {
                    self.copy_current_cell_to_clipboard();
                    self.clear_command_buffer();
                }
                _ => {
                    self.command_buffer.clear();
                    self.command_buffer.push('y');
                }
            },
            KeyCode::Char('p') if key.modifiers.is_empty() => {
                self.clear_command_buffer();
                self.paste_clipboard_into_cell();
            }
            KeyCode::Char('i') if key.modifiers.is_empty() => {
                self.clear_command_buffer();
                self.enter_insert_mode_at_start();
            }
            KeyCode::Char(':') if key.modifiers.is_empty() => {
                self.clear_command_buffer();
                self.enter_command_mode();
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                self.clear_command_buffer();
                self.enter_insert_mode_at_end();
            }
            KeyCode::Esc => {
                self.clear_command_buffer();
            }
            KeyCode::Enter => {
                self.move_cursor(0, 1);
            }
            _ => {
                self.clear_command_buffer();
            }
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.enter_normal_mode();
            }
            KeyCode::Enter => {
                self.enter_normal_mode();
                self.move_cursor(0, 1);
            }
            KeyCode::Left => self.move_edit_cursor_left(),
            KeyCode::Right => self.move_edit_cursor_right(),
            KeyCode::Backspace => self.backspace_cell_value(),
            KeyCode::Delete => self.delete_cell_value_forward(),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.insert_character_into_cell(c);
            }
            _ => {}
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.enter_normal_mode();
            }
            KeyCode::Enter => {
                self.execute_command();
                self.enter_normal_mode();
            }
            KeyCode::Backspace | KeyCode::Delete => {
                self.command_buffer.pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
    }

    fn execute_command(&mut self) {
        let binding = self.command_buffer.clone();
        let command: Vec<&str> = binding.split(' ').collect();

        match command[0] {
            "w" | "write" => {
                let path = command.get(1).copied().unwrap_or(self.file_name.as_str());
                self.handle_save_command(String::from(path));
            }
            "q" | "quit" => self.quit(),
            "wq" => {
                self.handle_save_command(self.file_name.clone());
                self.quit()
            }
            "cols" => {
                let amount = command.get(1).copied().unwrap_or("8");
                if let Ok(amount) = amount.parse::<usize>() {
                    self.visible_cols = amount;
                }
            }
            "theme" => {
                let name = command.get(1).copied();
                self.handle_theme_command(name);
            }
            _ => self.command_buffer = format!("unknown command: {}", command[0]),
        }
        self.clear_command_buffer();
    }

    fn handle_theme_command(&mut self, name: Option<&str>) {
        match name {
            Some(name) => match self.load_theme_by_name(name) {
                Ok(()) => self.command_buffer = format!("theme set to {}", name),
                Err(err) => self.command_buffer = format!("theme error: {}", err),
            },
            None => match self.list_available_themes() {
                Ok(themes) if themes.is_empty() => {
                    self.command_buffer = "no themes found".to_string();
                }
                Ok(themes) => {
                    self.command_buffer = format!("themes: {}", themes.join(", "));
                }
                Err(err) => self.command_buffer = format!("theme error: {}", err),
            },
        }
    }

    fn handle_save_command(&mut self, path: String) {
        match self.save_sheet(path.as_str()) {
            Ok(path) => self.command_buffer = format!("saved {}", path),
            Err(err) => self.command_buffer = format!("save failed: {}", err),
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

    fn go_to_first_row(&mut self) {
        self.cursor.row = 0;
        self.ensure_cursor_visible();
    }

    fn go_to_last_row_with_value(&mut self) {
        let last_row = self.cells.keys().map(|cell| cell.row).max().unwrap_or(0);
        self.cursor.row = last_row;
        self.ensure_cursor_visible();
    }

    fn insert_row_below_and_edit(&mut self) {
        let target = self.cursor.row.saturating_add(1);
        self.insert_row_at(target);
        self.cursor.row = target;
        self.cursor.col = 0;
        self.ensure_cursor_visible();
        self.enter_insert_mode_with_cursor(0);
    }

    fn insert_row_above_and_edit(&mut self) {
        let target = self.cursor.row;
        self.insert_row_at(target);
        self.cursor.col = 0;
        self.ensure_cursor_visible();
        self.enter_insert_mode_with_cursor(0);
    }

    fn insert_row_at(&mut self, row: usize) {
        let mut affected: Vec<(CellId, String)> = self
            .cells
            .iter()
            .filter(|(cell, _)| cell.row >= row)
            .map(|(cell, value)| (*cell, value.clone()))
            .collect();
        affected.sort_by(|a, b| match b.0.row.cmp(&a.0.row) {
            Ordering::Equal => b.0.col.cmp(&a.0.col),
            other => other,
        });

        for (cell, _) in &affected {
            self.cells.remove(cell);
        }

        for (cell, value) in affected {
            let new_cell = CellId::new(cell.row + 1, cell.col);
            self.cells.insert(new_cell, value);
        }
    }

    fn delete_current_row(&mut self) {
        let row = self.cursor.row;
        self.cells.retain(|cell, _| cell.row != row);

        let mut affected: Vec<(CellId, String)> = self
            .cells
            .iter()
            .filter(|(cell, _)| cell.row > row)
            .map(|(cell, value)| (*cell, value.clone()))
            .collect();
        affected.sort_by(|a, b| match a.0.row.cmp(&b.0.row) {
            Ordering::Equal => a.0.col.cmp(&b.0.col),
            other => other,
        });

        for (cell, _) in &affected {
            self.cells.remove(cell);
        }

        for (cell, value) in affected {
            let new_cell = CellId::new(cell.row - 1, cell.col);
            self.cells.insert(new_cell, value);
        }

        if self.cursor.row > 0 && !self.row_exists(self.cursor.row) {
            self.cursor.row = self.cursor.row.saturating_sub(1);
        }
        self.ensure_cursor_visible();
    }

    fn row_exists(&self, row: usize) -> bool {
        self.cells.keys().any(|cell| cell.row == row)
    }

    fn copy_current_cell_to_clipboard(&mut self) {
        let csv = self.cell_to_csv(self.cursor);
        self.clipboard = Some(csv.clone());
        self.command_buffer = format!("yy -> {}", csv);
    }

    fn paste_clipboard_into_cell(&mut self) {
        if let Some(clip) = self.clipboard.clone() {
            self.set_current_cell_value(clip.clone());
            self.command_buffer = "p".to_string();
        } else {
            self.command_buffer = "clipboard empty".to_string();
        }
    }

    fn enter_insert_mode_at_start(&mut self) {
        self.enter_insert_mode_with_cursor(0);
    }

    fn enter_insert_mode_at_end(&mut self) {
        let len = self.current_cell_value().len();
        self.enter_insert_mode_with_cursor(len);
    }

    fn enter_insert_mode_with_cursor(&mut self, cursor: usize) {
        let len = self.current_cell_value().len();
        self.mode = Mode::Insert(InsertState {
            cursor: cursor.min(len),
        });
        self.command_buffer.clear();
    }

    fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.command_buffer.clear();
    }

    fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_buffer.clear();
    }

    fn insert_character_into_cell(&mut self, ch: char) {
        let cursor = match self.mode {
            Mode::Insert(state) => state.cursor,
            Mode::Normal => return,
            Mode::Command => return,
        };

        let mut value = self.current_cell_value();
        let insert_at = cursor.min(value.len());
        value.insert(insert_at, ch);
        let new_cursor = insert_at + ch.len_utf8();
        self.set_current_cell_value(value);

        if let Mode::Insert(ref mut state) = self.mode {
            state.cursor = new_cursor;
        }
    }

    fn backspace_cell_value(&mut self) {
        let cursor = match self.mode {
            Mode::Insert(state) => state.cursor,
            Mode::Normal => return,
            Mode::Command => return,
        };

        if cursor == 0 {
            return;
        }

        let mut value = self.current_cell_value();
        let pos = cursor.min(value.len());
        if let Some((idx, ch)) = value[..pos].char_indices().next_back() {
            let end = idx + ch.len_utf8();
            value.drain(idx..end);
            self.set_current_cell_value(value);
            if let Mode::Insert(ref mut state) = self.mode {
                state.cursor = idx;
            }
        }
    }

    fn delete_cell_value_forward(&mut self) {
        let cursor = match self.mode {
            Mode::Insert(state) => state.cursor,
            Mode::Normal => return,
            Mode::Command => return,
        };

        let mut value = self.current_cell_value();
        let pos = cursor.min(value.len());
        if let Some((idx, ch)) = value[pos..].char_indices().next() {
            let start = pos + idx;
            let end = start + ch.len_utf8();
            value.drain(start..end);
            self.set_current_cell_value(value);
        }
    }

    fn move_edit_cursor_left(&mut self) {
        let cursor = match self.mode {
            Mode::Insert(state) => state.cursor,
            Mode::Normal => return,
            Mode::Command => return,
        };

        if cursor == 0 {
            return;
        }

        let value = self.current_cell_value();
        let pos = cursor.min(value.len());
        if let Some((idx, _)) = value[..pos].char_indices().next_back() {
            if let Mode::Insert(ref mut state) = self.mode {
                state.cursor = idx;
            }
        }
    }

    fn move_edit_cursor_right(&mut self) {
        let cursor = match self.mode {
            Mode::Insert(state) => state.cursor,
            Mode::Normal => return,
            Mode::Command => return,
        };

        let value = self.current_cell_value();
        let pos = cursor.min(value.len());
        let new_cursor = if let Some((idx, ch)) = value[pos..].char_indices().next() {
            pos + idx + ch.len_utf8()
        } else {
            value.len()
        };

        if let Mode::Insert(ref mut state) = self.mode {
            state.cursor = new_cursor;
        }
    }

    fn current_cell_value(&self) -> String {
        let id = CellId::new(self.cursor.row, self.cursor.col);
        self.cells.get(&id).cloned().unwrap_or_default()
    }

    fn set_current_cell_value(&mut self, value: String) {
        let id = CellId::new(self.cursor.row, self.cursor.col);
        if value.is_empty() {
            self.cells.remove(&id);
        } else {
            self.cells.insert(id, value);
        }
    }

    fn row_to_csv(&self, row: usize) -> String {
        let cols: Vec<usize> = self
            .cells
            .keys()
            .filter(|cell| cell.row == row)
            .map(|cell| cell.col)
            .collect();

        let max_col = cols.iter().copied().max().unwrap_or(0);
        let mut fields = Vec::with_capacity(max_col + 1);
        for col in 0..=max_col {
            let id = CellId::new(row, col);
            let value = self.cells.get(&id).cloned().unwrap_or_default();
            fields.push(csv_escape(&value));
        }
        fields.join(",")
    }

    fn cell_to_csv(&self, cursor: Cursor) -> String {
        let id = CellId::new(cursor.row, cursor.col);
        let value = self.cells.get(&id).cloned().unwrap_or_default();
        csv_escape(&value)
    }

    fn save_sheet(&self, path: &str) -> io::Result<String> {
        let mut file = File::create(&path)?;
        let max_row = self
            .cells
            .keys()
            .map(|cell| cell.row)
            .max()
            .unwrap_or(self.cursor.row);

        for row in 0..=max_row {
            let line = self.row_to_csv(row);
            writeln!(file, "{}", line)?;
        }
        file.flush()?;
        Ok(String::from(path))
    }

    fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }
}

fn apply_delta(value: usize, delta: i32) -> usize {
    if delta < 0 {
        value.saturating_sub((-delta) as usize)
    } else {
        value.saturating_add(delta as usize)
    }
}

fn csv_escape(value: &str) -> String {
    if value.contains(['"', ',', '\n']) {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}
