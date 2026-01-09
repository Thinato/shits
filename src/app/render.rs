use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::app::Mode;

use super::App;

const ROW_HEADER_WIDTH: u16 = 5;

impl App {
    pub(crate) fn render(&mut self, frame: &mut Frame) {
        if frame.area().is_empty() {
            return;
        }

        let total_area = frame.area();
        let global_style = self.global_style();
        frame.render_widget(Block::default().style(global_style), total_area);

        let desired_footer_lines: u16 = 2;
        let base_footer_height = desired_footer_lines.min(total_area.height);
        let max_grid_height = total_area.height.saturating_sub(base_footer_height);
        let cell_height: u16 = 1;
        let title_height: u16 = 1;
        let header_height: u16 = cell_height;
        let available_grid_height = max_grid_height.saturating_sub(title_height + header_height);
        let rows_to_render = (available_grid_height / cell_height) as usize;
        let grid_height_used =
            (title_height + header_height + cell_height.saturating_mul(rows_to_render as u16))
                .min(max_grid_height);
        let footer_carry = total_area
            .height
            .saturating_sub(grid_height_used + base_footer_height);
        let footer_height = base_footer_height + footer_carry;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(grid_height_used),
                Constraint::Length(footer_height),
            ])
            .split(total_area);

        let grid_area = layout[0];
        let footer_area = layout[1];

        self.visible_rows = rows_to_render;

        if grid_area.height > 0
            && grid_area.width > 0
            && rows_to_render > 0
            && self.visible_cols > 0
        {
            self.render_grid(frame, grid_area, rows_to_render, cell_height);
        }

        if footer_area.height > 0 {
            self.render_footer(
                frame,
                footer_area,
                base_footer_height as usize,
                footer_carry as usize,
            );
        }
    }

    fn render_grid(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        rows_to_render: usize,
        cell_height: u16,
    ) {
        if rows_to_render == 0 || cell_height == 0 || area.width == 0 {
            return;
        }

        let grid_title = Line::from("shits :: Terminal Sheet")
            .bold()
            .style(
                Style::default()
                    .fg(self.theme.title_fg)
                    .bg(self.theme.title_bg),
            )
            .centered();

        let grid_block = Block::default()
            .title(grid_title)
            .style(self.global_style());
        let inner_area = grid_block.inner(area);
        frame.render_widget(grid_block, area);

        if inner_area.height == 0 || inner_area.width == 0 {
            return;
        }

        let header_height = cell_height;
        if inner_area.height < header_height {
            return;
        }

        let row_header_width = ROW_HEADER_WIDTH.min(inner_area.width);

        let mut row_constraints = Vec::with_capacity(rows_to_render + 1);
        row_constraints.push(Constraint::Length(header_height));
        row_constraints
            .extend(std::iter::repeat(Constraint::Length(cell_height)).take(rows_to_render));
        let grid_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(inner_area);

        if grid_rows.is_empty() {
            return;
        }

        let column_header_row = grid_rows[0];
        self.render_column_headers(frame, column_header_row, row_header_width);

        for (row_idx, row_chunk) in grid_rows.iter().enumerate().skip(1) {
            let global_row = self.viewport.row + (row_idx - 1);
            self.render_data_row(frame, *row_chunk, row_header_width, global_row);
        }
    }

    fn render_column_headers(&self, frame: &mut Frame, area: Rect, row_header_width: u16) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let constraints: Vec<Constraint> = std::iter::once(Constraint::Length(row_header_width))
            .chain(
                std::iter::repeat(Constraint::Ratio(1, self.visible_cols.max(1) as u32))
                    .take(self.visible_cols),
            )
            .collect();

        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        if header_chunks.is_empty() {
            return;
        }

        // Top-left corner cell
        let corner_selected = self.cursor.row >= self.viewport.row
            && self.cursor.row < self.viewport.row + self.visible_rows
            && self.cursor.col >= self.viewport.col
            && self.cursor.col < self.viewport.col + self.visible_cols;
        let corner_style = self.header_style(corner_selected);
        let corner_widget = Paragraph::new("")
            .alignment(Alignment::Center)
            .block(Block::default().style(corner_style));
        frame.render_widget(corner_widget, header_chunks[0]);

        for (idx, chunk) in header_chunks.iter().enumerate().skip(1) {
            let global_col = self.viewport.col + idx - 1;
            let label = column_name(global_col);
            let selected = global_col == self.cursor.col;
            let style = self.header_style(selected);
            let widget = Paragraph::new(label)
                .alignment(Alignment::Center)
                .block(Block::default().style(style));
            frame.render_widget(widget, *chunk);
        }
    }

    fn render_data_row(
        &self,
        frame: &mut Frame,
        area: Rect,
        row_header_width: u16,
        global_row: usize,
    ) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let constraints: Vec<Constraint> = std::iter::once(Constraint::Length(row_header_width))
            .chain(
                std::iter::repeat(Constraint::Ratio(1, self.visible_cols.max(1) as u32))
                    .take(self.visible_cols),
            )
            .collect();

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        if col_chunks.is_empty() {
            return;
        }

        let row_label = (global_row + 1).to_string();
        let row_selected = global_row == self.cursor.row;
        let row_style = self.header_style(row_selected);
        let row_widget = Paragraph::new(row_label)
            .alignment(Alignment::Center)
            .block(Block::default().style(row_style));
        frame.render_widget(row_widget, col_chunks[0]);

        for (idx, cell_area) in col_chunks.iter().enumerate().skip(1) {
            let global_col = self.viewport.col + idx - 1;
            let style = if global_row == self.cursor.row && global_col == self.cursor.col {
                Style::default()
                    .bg(self.theme.selected_cell_bg)
                    .fg(self.theme.selected_cell_fg)
            } else if global_row == self.cursor.row {
                Style::default()
                    .bg(self.theme.selected_row_bg)
                    .fg(self.theme.selected_row_fg)
            } else if global_col == self.cursor.col {
                Style::default()
                    .bg(self.theme.selected_col_bg)
                    .fg(self.theme.selected_col_fg)
            } else {
                self.global_style()
            };

            let display = self.render_cell_text(global_row, global_col);

            let cell_block = Block::default().style(style);
            let cell_widget = Paragraph::new(display)
                .alignment(Alignment::Left)
                .block(cell_block);

            frame.render_widget(cell_widget, *cell_area);
        }
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect, base_lines: usize, carry_lines: usize) {
        if area.height == 0 {
            return;
        }

        let total_lines = (base_lines + carry_lines).max(1);
        let footer_constraints = vec![Constraint::Length(1); total_lines];
        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(footer_constraints)
            .split(area);

        if base_lines > 0 && !footer_chunks.is_empty() {
            let cell_label = format!("{}{}", column_name(self.cursor.col), self.cursor.row + 1);
            let line;
            if self.file_name.is_empty() {
                line = format!("[No Name] - [{}]", cell_label);
            } else {
                line = format!("{} - [{}]", self.file_name, cell_label);
            }
            frame.render_widget(
                Paragraph::new(line).style(self.global_style()),
                footer_chunks[0],
            );
        }

        if base_lines > 1 && footer_chunks.len() > 1 {
            let mode_line;
            match self.mode {
                Mode::Insert(_) => mode_line = format!("-- INSERT -- "),
                Mode::Command => mode_line = format!(":{}", self.command_buffer),
                Mode::Normal => mode_line = format!("{}", self.command_buffer),
            }
            frame.render_widget(
                Paragraph::new(mode_line).style(self.global_style()),
                footer_chunks[1],
            );
        }

        for chunk in footer_chunks.iter().skip(base_lines) {
            frame.render_widget(Paragraph::new("").style(self.global_style()), *chunk);
        }
    }

    fn render_cell_text(&self, row: usize, col: usize) -> Text<'static> {
        let value;
        let cursor = match self.mode {
            Mode::Insert(state) if self.cursor.row == row && self.cursor.col == col => {
                value = self.get_cell_value(row, col);
                Some(state.cursor)
            }
            _ => {
                value = self.get_cell_display_text(row, col);
                None
            }
        };

        if let Some(cursor) = cursor {
            let cursor = cursor.min(value.len());
            let (before, almost_after) = value.split_at(cursor);
            let cursor_style = Style::default()
                .fg(self.theme.cursor_fg)
                .bg(self.theme.cursor_bg);
            if almost_after.len() < 1 {
                return Text::from(Line::from(vec![
                    Span::raw(value),
                    Span::styled(" ", cursor_style),
                ]));
            }

            let (cursor_char, after) = almost_after.split_at(1);

            let line = Line::from(vec![
                Span::raw(before.to_string()),
                Span::styled(cursor_char.to_string(), cursor_style),
                Span::raw(after.to_string()),
            ]);
            Text::from(line)
        } else {
            Text::raw(value)
        }
    }

    fn get_cell_value(&self, row: usize, col: usize) -> String {
        match self.cells.get(&super::CellId::new(row, col)) {
            Some(value) if !value.is_empty() => value.clone(),
            _ => "".to_string(),
        }
    }

    fn get_cell_display_text(&self, row: usize, col: usize) -> String {
        let value = self.get_cell_value(row, col);

        if value.starts_with("=") {
            return "#NAME?".to_string();
        }

        return value;
    }

    fn header_style(&self, selected: bool) -> Style {
        if selected {
            Style::default()
                .bg(self.theme.header_selected_bg)
                .fg(self.theme.header_selected_fg)
        } else {
            Style::default()
                .bg(self.theme.header_bg)
                .fg(self.theme.header_fg)
        }
    }

    fn global_style(&self) -> Style {
        Style::default()
            .bg(self.theme.global_bg)
            .fg(self.theme.global_fg)
    }
}

fn column_name(mut index: usize) -> String {
    let mut name = String::new();
    index += 1;
    while index > 0 {
        let rem = (index - 1) % 26;
        name.insert(0, (b'A' + rem as u8) as char);
        index = (index - 1) / 26;
    }
    name
}
