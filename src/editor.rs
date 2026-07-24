use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::Path,
    process::exit,
    time::Duration,
};

use anyhow::Context;
use once_cell::sync::Lazy;

use crossterm::{
    self, cursor,
    event::{
        self, Event,
        KeyCode::{self, Backspace, Char, Enter, Esc},
        read,
    },
    queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, size},
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
    Command,
}

pub struct Editor {
    path_str: String,
    screen_w: usize,
    screen_h: usize,
    cursor_line: usize,
    cursor_y: usize,
    cursor_x: usize,
    cursor_x_rmind: usize,
    display_y: usize,
    file_entry: Vec<String>,
    display_lines: Vec<String>,
    syntaxer: Syntaxer,
    stdout: io::Stdout,
    lines: usize,
    mode: Mode,
    cmd: String,
}

impl Editor {
    pub fn new<'a>(path: &'a str) -> anyhow::Result<Self> {
        let mut buf = match File::open(path) {
            Ok(f) => BufReader::new(f),
            Err(_) => {
                let mut file = File::create(path)?;
                file.write_all(b" ")?;
                BufReader::new(file)
            }
        };

        let mut entry = String::new();

        buf.read_to_string(&mut entry)
            .context("Failed to read file entry to string")?;
        let mut file_entry: Vec<String> = entry.lines().map(String::from).collect();

        let (w, h) = size().context("Failed to get terminal size")?;

        if file_entry.len() <= 1 {
            for _ in 0..h - 4 {
                file_entry.push("".to_string());
            }
        }

        Ok(Editor {
            path_str: path.to_string(),
            file_entry: file_entry.clone(),
            display_lines: vec![],
            screen_w: w.into(),
            screen_h: h.into(),
            cursor_line: 0,
            cursor_y: 0,
            cursor_x: 0,
            cursor_x_rmind: 0,
            display_y: 0,
            syntaxer: Syntaxer::new(Path::new(path)),
            stdout: io::stdout(),
            lines: file_entry.len(),
            mode: Mode::Normal,
            cmd: String::from(":"),
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
            Mode::Normal => {
                self.normal_mode(key)?;
                return Ok(false);
            }
            Mode::Insert => {
                self.insert_mode(key)?;
                return Ok(false);
            }
            Mode::Command => {
                self.command_mode(key)?;
                return Ok(false);
            }
        }
    }

    pub fn normal_mode(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            Char('i') => {
                self.enter_insert_mode();
                return Ok(());
            }
            Char(':') => {
                self.enter_command_mode();
                return Ok(());
            }
            Char('j') => {
                self.move_cursor_down()?;
                return Ok(());
            }
            Char('k') => {
                self.move_cursor_up()?;
                return Ok(());
            }
            Char('l') => {
                self.move_cursor_right()?;
                return Ok(());
            }
            Char('h') => {
                self.move_cursor_left()?;
                return Ok(());
            }
            _ => return Ok(()),
        }
    }

    pub fn insert_mode(&mut self, key: KeyCode) -> anyhow::Result<()> {
        let mut current_line = self.file_entry[self.cursor_line].clone();

        if key == KeyCode::Backspace {
            self.backspace()?;
        }

        if key == KeyCode::Esc {
            self.mode = Mode::Normal;
        }
        let is_input = key.as_char();
        let input = match is_input {
            Some(ch) => ch,
            None => return Ok(()),
        };

        current_line.insert(self.cursor_x, input);
        self.cursor_x += 1;
        self.cursor_x_rmind += 1;
        self.file_entry[self.cursor_line] = current_line.clone();

        if self.display_lines.is_empty() {
            self.display_lines
                .push(self.syntaxer.highlight_line(&current_line)?);
        }
        self.display_lines[self.cursor_y] = self.syntaxer.highlight_line(&current_line)?;

        Ok(())
    }

    pub fn backspace(&mut self) -> anyhow::Result<()> {
        let mut current_line = self.file_entry[self.cursor_line].clone();
        if self.cursor_x > 0 {
            current_line.remove(self.cursor_x - 1);
            self.file_entry[self.cursor_line] = current_line.clone();
            self.display_lines[self.cursor_y] =
                self.syntaxer.highlight_line(&current_line.clone())?;
            self.cursor_x -= 1;
            self.cursor_x_rmind -= 1;
        }
        self.display()?;

        Ok(())
    }

    pub fn command_mode(&mut self, key: KeyCode) -> anyhow::Result<()> {
        match key {
            Enter => {
                if self.cmd == ":q".to_string() {
                    exit(0);
                } else if self.cmd == ":wq".to_string() {
                    self.overwrite_file()?;
                    exit(0);
                }
                self.mode = Mode::Normal;
                self.cmd = ":".to_string();
            }
            Backspace => {
                self.cmd.pop();
            }
            Esc => {
                self.mode = Mode::Normal;
                self.cmd = ":".to_string();
            }
            _ => {}
        }
        match key.as_char() {
            Some(ch) => {
                self.cmd.push(ch);
                return Ok(());
            }
            None => return Ok(()),
        }
    }

    pub fn display(&mut self) -> anyhow::Result<()> {
        self.clear_screen()?;
        self.display_lines()?;
        self.display_pos()?;
        self.display_mode()?;
        self.display_cursor()?;
        self.display_command_line()?;

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
        disable_raw_mode()?;
        self.stdout.flush()?;
        Ok(())
    }

    pub fn display_lines(&mut self) -> anyhow::Result<()> {
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
        Ok(())
    }

    pub fn display_pos(&mut self) -> anyhow::Result<()> {
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
        Ok(())
    }

    pub fn display_mode(&mut self) -> anyhow::Result<()> {
        queue!(
            self.stdout,
            cursor::MoveTo(
                0,
                u16::try_from(self.screen_h).expect("Your monitor is probably mega large")
            ),
            Print(match self.mode {
                Mode::Normal => "normal",
                Mode::Insert => "INSERT",
                _ => "",
            })
        )?;
        Ok(())
    }

    pub fn display_cursor(&mut self) -> anyhow::Result<()> {
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

    pub fn enter_command_mode(&mut self) {
        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Command;
            }
            Mode::Insert => {}
            Mode::Command => {
                self.mode = Mode::Normal;
            }
        }
    }

    pub fn enter_insert_mode(&mut self) {
        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Insert;
            }
            Mode::Insert => {}
            _ => {}
        }
    }

    pub fn move_cursor_right(&mut self) -> anyhow::Result<()> {
        let current_line_len = self.file_entry[self.cursor_line].len();
        if self.cursor_x < self.screen_w {
            if self.cursor_x < current_line_len {
                self.cursor_x += 1;
                self.cursor_x_rmind += 1;
            }
            return Ok(());
        } else {
            return Ok(());
        }
    }

    pub fn move_cursor_left(&mut self) -> anyhow::Result<()> {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.cursor_x_rmind -= 1;
            return Ok(());
        } else {
            return Ok(());
        }
    }

    pub fn move_cursor_up_x(&mut self) -> anyhow::Result<()> {
        let prev_line = self.file_entry[self.cursor_line - 1].len();
        if self.cursor_x_rmind > prev_line {
            self.cursor_x = prev_line;
        } else {
            self.cursor_x = self.cursor_x_rmind;
        }
        Ok(())
    }

    pub fn move_cursor_up(&mut self) -> anyhow::Result<()> {
        if self.cursor_line == 0 {
            return Ok(());
        }

        if self.cursor_y > 0 {
            self.move_cursor_up_x()?;
            self.cursor_line -= 1;
            self.cursor_y -= 1;
            return Ok(());
        } else {
            self.move_cursor_up_x()?;

            self.display_y -= 1;
            self.cursor_line -= 1;
            self.scroll_up()?;
            self.clear_screen()?;
            return Ok(());
        }
    }

    pub fn move_cursor_down_x(&mut self) -> anyhow::Result<()> {
        if self.cursor_line == self.file_entry.len() - 1 {
            return Ok(());
        }
        let next_line_len = self.file_entry[self.cursor_line + 1].len();
        if self.cursor_x_rmind > next_line_len {
            self.cursor_x = next_line_len;
        } else {
            self.cursor_x = self.cursor_x_rmind;
        }
        Ok(())
    }

    pub fn move_cursor_down(&mut self) -> anyhow::Result<()> {
        if self.cursor_line == self.lines {
            return Ok(());
        }
        if self.cursor_y < self.screen_h - 2 {
            self.move_cursor_down_x()?;
            self.cursor_y += 1;
            self.cursor_line += 1;
            return Ok(());
        } else {
            if self.cursor_line == self.file_entry.len() - 1 {
                return Ok(());
            }

            self.display_y += 1;
            self.cursor_line += 1;
            self.move_cursor_down_x()?;
            self.scroll_down()?;
            self.clear_screen()?;
            return Ok(());
        }
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

    pub fn display_command_line(&mut self) -> anyhow::Result<()> {
        match self.mode {
            Mode::Command => {
                queue!(
                    self.stdout,
                    cursor::MoveTo(0, u16::try_from(self.screen_h).expect("Err")),
                    terminal::Clear(terminal::ClearType::CurrentLine),
                    Print(self.cmd.clone()),
                )?;
                self.stdout.flush()?;
                Ok(())
            }
            _ => Ok(()),
        }
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

    pub fn overwrite_file(&mut self) -> anyhow::Result<()> {
        let file = File::create(self.path_str.clone())?;
        let mut writer = BufWriter::new(file);

        for line in &self.file_entry {
            writer.write_all(line.as_bytes())?;
            writer.write_all(b"\n")?;
        }

        Ok(())
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
        for line in self.display_lines.iter().clone() {
            let syntax_line = self.syntaxer.highlight_line(&line)?;
            syntaxed_lines.push(syntax_line);
        }

        self.display_lines = syntaxed_lines;
        if self.display_lines.len() == 0 {
            for _ in 0..self.screen_h - 1 {
                self.display_lines.push("".to_string());
                self.file_entry.push("".to_string());
            }
        }
        Ok(())
    }
}
