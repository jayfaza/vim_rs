use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
    time::Duration,
};

use once_cell::sync::Lazy;

use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode::Char, read},
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
    Lazy::new(|| ThemeSet::load_defaults().themes["base16-ocean.dark"].clone());

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
    pub fn highlight_line(&mut self, line: &str) -> String {
        let ranges = self.h.highlight_line(line, &self.ss).unwrap();
        as_24_bit_terminal_escaped(&ranges[..], false)
    }
}

pub struct Editor<'a> {
    path_str: String,
    path: &'a Path,
    bufreader: BufReader<File>,
    screen_w: usize,
    screen_h: usize,
    cursor_line: usize,
    cursor_y: usize,
    cursor_x: usize,
    display_y: usize,
    file_entry: Vec<String>,
    display_lines: Vec<String>,
    syntaxer: Syntaxer,
    stdout: io::Stdout,
    lines: usize,
}

impl<'a> Editor<'a> {
    pub fn new(path: &'a str) -> Self {
        let mut buf = BufReader::new(File::open(path).unwrap());
        let mut string = String::new();
        buf.read_to_string(&mut string).unwrap();
        let file_entry: Vec<String> = string.lines().map(String::from).collect();
        let (w, h) = size().unwrap();

        Editor {
            path_str: path.to_string(),
            path: Path::new(path),
            bufreader: buf,
            file_entry: file_entry.clone(),
            display_lines: vec!(),
            screen_w: w.into(),
            screen_h: h.into(),
            cursor_line: 0,
            cursor_y: 0,
            cursor_x: 0,
            display_y: 0,
            syntaxer: Syntaxer::new(Path::new(path)),
            stdout: io::stdout(),
            lines: file_entry.iter().count(),
        }
    }

    pub fn update_terminal_size(&mut self) -> io::Result<()> {
        let (w, h) = size()?;
        self.screen_w = w.into();
        self.screen_h = h.into();
        Ok(())
    }

    pub fn handle_event(&mut self, key: char) -> bool {
        match key {
            'j' => {
                if self.cursor_line == self.lines {
                    return false;
                }
                if self.cursor_y < self.screen_h {
                    self.cursor_y += 1;
                    self.cursor_line += 1;
                    return false;
                } else {
                    self.display_y += 1;
                    self.cursor_line += 1;
                    self.clear_screen();
                    return false;
                }
            }
            'k' => {
                if self.cursor_line == 0 {
                    return false;
                }

                if self.cursor_y > 0 {
                    self.cursor_line -= 1;
                    self.cursor_y -= 1;
                    return false;
                } else {
                    self.display_y -= 1;
                    self.cursor_line -= 1;
                    self.clear_screen();
                    return false;
                }
            }
            'l' => {
                if self.cursor_x < self.screen_w {
                    self.cursor_x += 1;
                    return false;
                } else {
                    return false;
                }
            }
            'h' => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    return false;
                } else {
                    return false;
                }
            }
            'q' => {
                return true;
            }
            _ => return false,
        }
    }

    pub fn clear_screen(&mut self) {
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    }

    pub fn pick_event(&mut self) -> io::Result<Option<char>> {
        match read()? {
            Event::Key(event) => match event.code {
                Char('j') => return Ok(Some('j')),
                Char('k') => return Ok(Some('k')),
                Char('h') => return Ok(Some('h')),
                Char('l') => return Ok(Some('l')),
                Char('q') => return Ok(Some('q')),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }
    
    pub fn display_lines(&mut self) {
        let range = self.display_y + self.screen_h;
        let bound = match range {
            range if range > self.lines => self.lines,
            range if range <= self.lines => range,
            _ => self.lines,
        };

        self.display_lines = self.file_entry[self.display_y..bound].to_vec();
    }

    pub fn display(&mut self) -> io::Result<()> {
        // let display_lines = &self.file_entry[self.display_y..self.display_y + self.screen_h];
        self.display_lines();

        for (offset, line) in self.display_lines.iter().enumerate() {
            let syntaxed_line = self.syntaxer.highlight_line(line);
            queue!(
                self.stdout,
                cursor::MoveTo(0, u16::try_from(offset).expect("Large")),
                Print(syntaxed_line)
            )?;
        }

        queue!(
            self.stdout,
            cursor::MoveTo(0, u16::try_from(self.screen_h).unwrap()),
            terminal::Clear(terminal::ClearType::CurrentLine),
            Print(format!("{}", self.cursor_line))
        )?;

        queue!(
            self.stdout,
            cursor::MoveTo(
                u16::try_from(self.cursor_x).expect("Large"),
                u16::try_from(self.cursor_y).expect("Large")
            )
        )?;

        Ok(())
    }

    pub fn editor_start(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        // queue!(self.stdout, terminal::EnterAlternateScreen)?;
        queue!(self.stdout, terminal::Clear(terminal::ClearType::All))?;
        self.display()?;
        self.stdout.flush()?;
        loop {
            if event::poll(Duration::from_millis(50))? {
                match self.pick_event()? {
                    Some(e) => {
                        let exit = self.handle_event(e);
                        if exit {
                            queue!(self.stdout, terminal::Clear(terminal::ClearType::All))?;
                            break;
                        }
                        self.display()?;
                        self.stdout.flush()?;
                    }
                    None => {}
                }
            }
        }
        // queue!(self.stdout, LeaveAlternateScreen)?;
        self.stdout.flush()?;
        disable_raw_mode()?;
        Ok(())
    }
}
