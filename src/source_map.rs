use std::path::PathBuf;

pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
}

#[derive(Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn add(&mut self, path: PathBuf, text: String) -> u32 {
        let id = u32::try_from(self.files.len()).unwrap_or(u32::MAX);
        self.files.push(SourceFile { path, text });
        id
    }

    #[must_use]
    pub fn get(&self, file: u32) -> &SourceFile {
        &self.files[file as usize]
    }
}
