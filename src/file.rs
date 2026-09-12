use std::fs;
use anyhow::Context;

#[derive(Debug)]
pub struct OpenFile {
    pub content: Vec<String>,
    pub length: usize,
}

impl OpenFile {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let file_entry = OpenFile::open_file(path)?;
        let length = file_entry.len();
        Ok(Self {
            content: file_entry,
            length,
        })
    }

    fn open_file(path: &str) -> anyhow::Result<Vec<String>> {
        Ok(fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path))?
            .lines()
            .map(|e| e.to_string())
            .collect())
    }
}
