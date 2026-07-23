use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
    time::Duration,
};

use anyhow::Context;
use once_cell::sync::Lazy;

use crossterm::{
    self, cursor, event::{self, Event, KeyCode::{self, Char}, read}, queue, style::Print, terminal::{self, disable_raw_mode, enable_raw_mode, size},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{Theme, ThemeSet},
    parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};

static SYNTAXSET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME: Lazy<Theme> =
    Lazy::new(|| ThemeSet::load_defaults().themes["base16-eighties.dark"].clone());

pub struct Syntaxer {
    ss: SyntaxSet,
    h: HighlightLines<'static>,
}

impl Syntaxer {
    pub fn new(path: &Path) -> Self {
        let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        let syntax = SYNTAXSET
            .find_syntax_by_extension(ext)
            .unwrap_or(SYNTAXSET.find_syntax_plain_text());
        let h = HighlightLines::new(syntax, &THEME);
        Syntaxer {
            ss: SYNTAXSET.clone(),
            h: h,
        }
    }
    pub fn highlight_line(&mut self, line: &str) -> anyhow::Result<String> {
        let ranges = self
            .h
            .highlight_line(line, &self.ss)
            .context("Failed to highlight line.")?;
        Ok(as_24_bit_terminal_escaped(&ranges[..], false))
    }
}

pub enum Mode {
    Normal,
    Insert,
}

pub struct Editor<'a> {
    #[warn(dead_code)]
    path_str: String,
    #[warn(dead_code)]
    path: &'a Path,
    #[warn(dead_code)]
    bufreader: BufReader<File>,
    screen_w: usize,
    screen_h: usize,
    cursor_line: usize,
    cursor_y: usize,
    cursor_x: usize,
    cursor_x_rmind: Vec<usize>,
    display_y: usize,
    file_entry: Vec<String>,
    display_lines: Vec<String>,
    syntaxer: Syntaxer,
    stdout: io::Stdout,
    lines: usize,
    mode: Mode,

}

impl<'a> Editor<'a> {
    pub fn new(path: &'a str) -> anyhow::Result<Self> {
        let mut buf = BufReader::new(File::open(path).context("Failed to create BufReader")?);
        let mut string = String::new();
        buf.read_to_string(&mut string)
            .context("Failed to read file entry to string")?;
        let file_entry: Vec<String> = string.lines().map(String::from).collect();
        let (w, h) = size().context("Failed to get terminal size")?;

        Ok(Editor {
            path_str: path.to_string(),
            path: Path::new(path),
            bufreader: buf,
            file_entry: file_entry.clone(),
            display_lines: vec![],
            screen_w: w.into(),
            screen_h: h.into(),
            cursor_line: 0,
            cursor_y: 0,
            cursor_x: 0,
            cursor_x_rmind: vec![],
            display_y: 0,
            syntaxer: Syntaxer::new(Path::new(path)),
            stdout: io::stdout(),
            lines: file_entry.iter().count(),
            mode: Mode::Normal,
        })
    }

    pub fn update_terminal_size(&mut self) -> anyhow::Result<()> {
        let (w, h) = size().context("Failed to get terminal size")?;
        self.screen_w = w.into();
        self.screen_h = h.into();
        Ok(())
    }

    pub fn handle_event(&mut self, key: KeyCode) -> anyhow::Result<bool> {
        match self.mode {
            Mode::Normal => {},
            Mode::Insert => {
                let mut current_line = self.file_entry[self.cursor_line].clone();
                let input = key.as_char().unwrap_or(' ');
                if input == ' ' {
                    self.mode = Mode::Normal;
                    return Ok(false);
                }
                current_line.insert(self.cursor_x, input);
                self.cursor_x += 1;
                self.file_entry[self.cursor_line] = current_line.clone();

                self.display_lines[self.cursor_y] = self.syntaxer.highlight_line(&current_line)?;
                // self.get_displayable_line()?;
                return Ok(false);
            }
        }
        match key {
            Char('i') => {
                match self.mode {
                    Mode::Normal => {
                        self.mode = Mode::Insert;
                        return Ok(false);
                    },
                    Mode::Insert => {
                        return Ok(false);
                    }
                } 
            }
            Char('j') => {
                if self.cursor_line == self.lines {
                    return Ok(false);
                }
                if self.cursor_y < self.screen_h - 2 {
                    if self.file_entry[self.cursor_line + 1].len() < self.cursor_x {
                        self.cursor_x_rmind.push(self.cursor_x);
                        self.cursor_x = self.file_entry[self.cursor_line + 1].len();
                    } else {
                        let next_line = self.file_entry[self.cursor_line + 1].len();
                        for x in self.cursor_x_rmind.clone() {
                            if x <= next_line {
                                if x > self.cursor_x {
                                    self.cursor_x = x;
                                }
                            }
                        }
                    }
                    self.cursor_y += 1;
                    self.cursor_line += 1;
                    return Ok(false);
                } else {
                    if self.cursor_line == self.file_entry.len() - 1 {
                        return Ok(false);
                    }
                    if self.file_entry[self.cursor_line + 1].len() < self.cursor_x {
                        self.cursor_x_rmind.push(self.cursor_x);
                        self.cursor_x = self.file_entry[self.cursor_line + 1].len();
                    } else {
                        let next_line = self.file_entry[self.cursor_line + 1].len();
                        for x in self.cursor_x_rmind.clone() {
                            if x <= next_line {
                                if x > self.cursor_x {
                                    self.cursor_x = x;
                                }
                            }
                        }
                    }                   
                    self.display_y += 1;
                    self.cursor_line += 1;
                    self.scroll_down()?;
                    self.clear_screen()?;
                    return Ok(false);
                }
            }
            Char('k') => {
                if self.cursor_line == 0 {
                    return Ok(false);
                }

                if self.cursor_y > 0 {
                    let prev_line = self.file_entry[self.cursor_line - 1].len();
                    if self.cursor_x > prev_line {
                        self.cursor_x_rmind.push(self.cursor_x);
                        self.cursor_x = prev_line;
                    } else {
                        for x in self.cursor_x_rmind.clone() {
                            if x <= prev_line {
                                if x > self.cursor_x {
                                    self.cursor_x = x;
                                }
                            }
                        }
                    }
                    self.cursor_line -= 1;
                    self.cursor_y -= 1;
                    return Ok(false);
                } else {
                    let prev_line = self.file_entry[self.cursor_line - 1].len();
                    if self.cursor_x > prev_line {
                        self.cursor_x_rmind.push(self.cursor_x);
                        self.cursor_x = prev_line;
                    } else {
                        for x in self.cursor_x_rmind.clone() {
                            if x <= prev_line {
                                if x > self.cursor_x {
                                    self.cursor_x = x;
                                }
                            }
                        }
                    }
                    
                    self.display_y -= 1;
                    self.cursor_line -= 1;
                    self.scroll_up()?;
                    self.clear_screen()?;
                    return Ok(false);
                }
            }
            Char('l') => {
                if self.cursor_x < self.screen_w {
                    if self.cursor_x < self.file_entry[self.cursor_line].len() {
                        self.cursor_x += 1;
                    }
                    return Ok(false);
                } else {
                    return Ok(false);
                }
            }
            Char('h') => {
                if self.cursor_x > 0 {
                    
                    self.cursor_x -= 1;
                    return Ok(false);
                } else {
                    return Ok(false);
                }
            }
            Char('q') => {
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }

    pub fn clear_screen(&mut self) -> anyhow::Result<()> {
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All))
            .context("Failed to clear screen")?;
        Ok(())
    }

    pub fn pick_event(&mut self) -> anyhow::Result<Option<KeyCode>> {
        match read().context("Failed to read events")? {
            Event::Key(event) => Ok(Some(event.code)),
            _ => Ok(None),
        }
    }

    pub fn calc_bound(&self) -> usize {
        let range = self.display_y + self.screen_h;
        let bound = match range {
            range if range > self.lines => self.lines - 1,
            range if range <= self.lines => range - 1,
            _ => self.lines - 1,
        };
        bound
    }

    pub fn get_displayable_line(&mut self) -> anyhow::Result<()> {
        let bound = self.calc_bound();

        self.display_lines = self.file_entry[self.display_y..bound].to_vec();
        let mut syntaxed_lines: Vec<String> = Vec::new();
        for line in self.display_lines.clone() {
            let syntax_line = self.syntaxer.highlight_line(&line)?;
            syntaxed_lines.push(syntax_line);
        }
        self.display_lines = syntaxed_lines;
        Ok(())
    }

    pub fn scroll_down(&mut self) -> anyhow::Result<()> {
        self.display_lines.remove(0);

        let bound = self.calc_bound();

        let next_syntaxed_line =
            self.syntaxer
                .highlight_line(self.file_entry.get(bound - 1).context(format!(
                    "Indexation failure with highlighted lines: No element with index {}",
                    bound - 1
                ))?)?;

        self.display_lines.push(next_syntaxed_line);
        Ok(())
    }

    pub fn scroll_up(&mut self) -> anyhow::Result<()> {
        self.display_lines.pop();

        let prev_syntaxed_line =
            self.syntaxer
                .highlight_line(self.file_entry.get(self.cursor_line).context(format!(
                    "Failed to get element with index: {}",
                    self.cursor_line
                ))?)?;

        self.display_lines.insert(0, prev_syntaxed_line);
        Ok(())
    }

    pub fn display(&mut self) -> anyhow::Result<()> {
        for (offset, line) in self.display_lines.iter().enumerate() {
            queue!(
                self.stdout,
                cursor::MoveTo(
                    0,
                    u16::try_from(offset).expect("Your monitor is mega large")
                ),
                Print(line)
            )?;
        }

        queue!(
            self.stdout,
            cursor::MoveTo(
                u16::try_from(
                    self.screen_w - format!("{}/{}", self.cursor_line, self.cursor_x).len()
                )
                .expect("Your monitor is mega large"),
                u16::try_from(self.screen_h).expect("Your monitor is probably mega large"),
            ),
            Print(format!("{}/{}", self.cursor_line, self.cursor_x))
        )
        .context("Failed to display current cursor line and cursor_x")?;

        queue!(self.stdout, 
            cursor::MoveTo(
                0, u16::try_from(self.screen_h).expect("Your monitor is probably mega large")
            ),
            Print(match self.mode {
                Mode::Normal => "normal",
                Mode::Insert => "INSERT",
            })
            )?;

        queue!(
            self.stdout,
            cursor::MoveTo(
                u16::try_from(self.cursor_x).expect("Large"),
                u16::try_from(self.cursor_y).expect("Large")
            )
        )
        .context("Failed to move cursor to his place")?;

        Ok(())
    }

    pub fn editor_start(&mut self) -> anyhow::Result<()> {
        enable_raw_mode()?;
        self.clear_screen()?;
        self.get_displayable_line()?;
        self.display()?;
        self.stdout.flush()?;

        loop {
            if event::poll(Duration::from_millis(50)).context("Failure with event poll")? {
                match self.pick_event()? {
                    Some(e) => {
                        let exit = self.handle_event(e)?;
                        if exit {
                            break;
                        }
                        self.display()?;
                        self.stdout.flush()?;
                    }
                    None => {}
                }
            }
        }
        self.clear_screen()?;
        disable_raw_mode()?;
        self.stdout.flush()?;
        Ok(())
    }
}
