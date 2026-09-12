use crossterm::terminal::size;
pub struct Screen {
    pub w: u16,
    pub h: u16,
}

impl Screen {
    pub fn new() -> anyhow::Result<Self> {
        let (w, h) = size()?;
        Ok(Self { w, h })
    }
}
