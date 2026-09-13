use anyhow::Context;
use std::{fs::{self, OpenOptions}, io::Write};

#[derive(Debug)]
pub struct OpenFile {
    pub content: Vec<String>,
    pub path: String,
    pub length: usize,
}

impl OpenFile {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file_entry = OpenFile::open_file(path)?;
        let length = file_entry.len();
        Ok(Self {
            content: file_entry,
            length,
            path: path.to_string(),
        })
    }

    fn open_file(path: &str) -> anyhow::Result<Vec<String>> {
        Ok(fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path))?
            .lines()
            .map(|e| e.to_string())
            .collect())
    }

    pub fn write(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.path).context("Failed to write file")?;
        let mut buf = String::new();
        for (idx, string) in self.content.iter().enumerate() {
            if self.content.get(idx + 1).is_some() {
                buf.push_str(string);
                buf.push('\n');
            } else {
                buf.push_str(string);
            }
        }
        file.write(buf.as_bytes())?;
        Ok(())
    }
}
