mod events;
mod keymap;
mod render;

use std::{collections::HashMap, fmt};

use color_eyre::Result;
use ratatui::DefaultTerminal;

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
            Mode::Command => write!(f, ":"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct InsertState {
    cursor: usize,
}
