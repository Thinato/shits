mod events;
mod keymap;
mod render;

use std::{
    collections::HashMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use ratatui::{DefaultTerminal, style::Color};
use serde::Deserialize;

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

    fn load_theme_by_name(&mut self, name: &str) -> Result<(), String> {
        let file_name = normalize_theme_name(name)?;
        let path = themes_dir().join(file_name);
        let raw = fs::read_to_string(&path)
            .map_err(|err| format!("unable to read {}: {}", path.display(), err))?;
        let config: ThemeConfig =
            serde_json::from_str(&raw).map_err(|err| format!("invalid theme json: {}", err))?;
        let mut theme = Theme::default();
        config.apply_to(&mut theme)?;
        self.theme = theme;
        Ok(())
    }

    fn list_available_themes(&self) -> Result<Vec<String>, String> {
        let entries = fs::read_dir(themes_dir())
            .map_err(|err| format!("unable to read themes dir: {}", err))?;
        let mut themes = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|err| format!("unable to read theme entry: {}", err))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                themes.push(stem.to_string());
            }
        }
        themes.sort();
        Ok(themes)
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

#[derive(Debug, Default, Deserialize)]
struct ThemeConfig {
    global_bg: Option<[u8; 3]>,
    global_fg: Option<[u8; 3]>,
    cursor_fg: Option<[u8; 3]>,
    cursor_bg: Option<[u8; 3]>,
    title_fg: Option<[u8; 3]>,
    title_bg: Option<[u8; 3]>,
    header_fg: Option<[u8; 3]>,
    header_bg: Option<[u8; 3]>,
    header_selected_fg: Option<[u8; 3]>,
    header_selected_bg: Option<[u8; 3]>,
    selected_cell_fg: Option<[u8; 3]>,
    selected_cell_bg: Option<[u8; 3]>,
    selected_row_fg: Option<[u8; 3]>,
    selected_row_bg: Option<[u8; 3]>,
    selected_col_fg: Option<[u8; 3]>,
    selected_col_bg: Option<[u8; 3]>,
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

impl ThemeConfig {
    fn apply_to(self, theme: &mut Theme) -> Result<(), String> {
        apply_color(&mut theme.global_bg, self.global_bg)?;
        apply_color(&mut theme.global_fg, self.global_fg)?;
        apply_color(&mut theme.cursor_fg, self.cursor_fg)?;
        apply_color(&mut theme.cursor_bg, self.cursor_bg)?;
        apply_color(&mut theme.title_fg, self.title_fg)?;
        apply_color(&mut theme.title_bg, self.title_bg)?;
        apply_color(&mut theme.header_fg, self.header_fg)?;
        apply_color(&mut theme.header_bg, self.header_bg)?;
        apply_color(&mut theme.header_selected_fg, self.header_selected_fg)?;
        apply_color(&mut theme.header_selected_bg, self.header_selected_bg)?;
        apply_color(&mut theme.selected_cell_fg, self.selected_cell_fg)?;
        apply_color(&mut theme.selected_cell_bg, self.selected_cell_bg)?;
        apply_color(&mut theme.selected_row_fg, self.selected_row_fg)?;
        apply_color(&mut theme.selected_row_bg, self.selected_row_bg)?;
        apply_color(&mut theme.selected_col_fg, self.selected_col_fg)?;
        apply_color(&mut theme.selected_col_bg, self.selected_col_bg)?;
        Ok(())
    }
}

fn apply_color(target: &mut Color, value: Option<[u8; 3]>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    *target = Color::Rgb(value[0], value[1], value[2]);
    Ok(())
}

fn themes_dir() -> PathBuf {
    PathBuf::from("themes")
}

fn normalize_theme_name(name: &str) -> Result<String, String> {
    let path = Path::new(name);
    if path.components().count() > 1 {
        return Err("theme name must be a file name".to_string());
    }
    let raw = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "invalid theme name".to_string())?;
    if raw.ends_with(".json") {
        Ok(raw.to_string())
    } else {
        Ok(format!("{}.json", raw))
    }
}
