use anyhow::Context;
use std::fs;

pub fn exists_or_create(path: &str) -> anyhow::Result<()> {
    let path = std::path::Path::new(path);

    if path.exists() {
        Ok(())
    } else {
        fs::File::create(path).context(format!("Failed to create file: {:?}", path))?;
        Ok(())
    }
}

pub fn is_dir(path: &str) -> bool {
    std::path::Path::new(path).is_dir()
}
