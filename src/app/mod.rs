mod events;
mod keymap;
mod render;

use std::{collections::HashMap, fmt};

use color_eyre::Result;
use ratatui::{DefaultTerminal, style::Color};

const DEFAULT_VISIBLE_ROWS: usize = 12;
const DEFAULT_VISIBLE_COLS: usize = 8;

#[derive(Debug)]
pub struct App {
    running: bool,
    mode: Mode,
    visible_rows: usize,
    visible_cols: usize,
    viewport: Viewport,
    cells: HashMap<CellId, String>,
    cursor: Cursor,
    file_name: String,
    command_buffer: String,
    clipboard: Option<String>,
    theme: Theme,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: false,
            mode: Mode::Normal,
            visible_rows: DEFAULT_VISIBLE_ROWS,
            visible_cols: DEFAULT_VISIBLE_COLS,
            viewport: Viewport::default(),
            cells: HashMap::new(),
            cursor: Cursor::default(),
            file_name: String::from("Untitled.csv"),
            command_buffer: String::new(),
            clipboard: None,
            theme: Theme::default(),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    pub(crate) fn quit(&mut self) {
        self.running = false;
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    row: usize,
    col: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct Viewport {
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CellId {
    row: usize,
    col: usize,
}

impl CellId {
    const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Normal,
    Insert(InsertState),
    Command,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert(_) => write!(f, "INSERT"),
            Mode::Command => write!(f, "COMMAND"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InsertState {
    cursor: usize,
}

#[derive(Debug)]
struct Theme {
    global_bg: Color,
    global_fg: Color,
    cursor_fg: Color,
    cursor_bg: Color,
    title_fg: Color,
    title_bg: Color,
    header_fg: Color,
    header_bg: Color,
    header_selected_fg: Color,
    header_selected_bg: Color,
    selected_cell_fg: Color,
    selected_cell_bg: Color,
    selected_row_fg: Color,
    selected_row_bg: Color,
    selected_col_fg: Color,
    selected_col_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            global_fg: Color::Rgb(255, 255, 255),
            global_bg: Color::Rgb(18, 18, 18),
            cursor_fg: Color::Rgb(158, 149, 199),
            cursor_bg: Color::Rgb(18, 18, 18),
            title_fg: Color::Rgb(158, 149, 199),
            title_bg: Color::Rgb(18, 18, 18),
            header_fg: Color::Rgb(255, 255, 255),
            header_bg: Color::Rgb(18, 18, 18),
            header_selected_fg: Color::Rgb(255, 255, 255),
            header_selected_bg: Color::Rgb(70, 70, 70),
            selected_cell_fg: Color::Rgb(0, 0, 0),
            selected_cell_bg: Color::Rgb(255, 221, 51),
            selected_row_fg: Color::Rgb(255, 255, 255),
            selected_row_bg: Color::Rgb(32, 32, 32),
            selected_col_fg: Color::Rgb(255, 255, 255),
            selected_col_bg: Color::Rgb(32, 32, 32),
        }
    }
}
