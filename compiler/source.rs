//! Immutable original-byte snapshots. Coordinates never normalize source.
use std::{
    fs::File,
    io::{self, Read},
    path::Path,
    sync::Arc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(pub usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub source: SourceId,
    pub start: usize,
    pub end: usize,
}
impl Span {
    pub fn new(source: SourceId, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }
}
#[derive(Debug)]
pub struct SourceFile {
    pub id: SourceId,
    pub name: String,
    data: Result<String, (Arc<[u8]>, std::str::Utf8Error)>,
    lines: Vec<usize>,
}
impl SourceFile {
    pub fn bytes(&self) -> &[u8] {
        match &self.data {
            Ok(s) => s.as_bytes(),
            Err((bytes, _)) => bytes,
        }
    }
    pub fn text(&self) -> Result<&str, std::str::Utf8Error> {
        match &self.data {
            Ok(s) => Ok(s),
            Err((_, e)) => Err(*e),
        }
    }
    pub fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.id, start, end)
    }
    pub fn slice(&self, span: Span) -> Option<&str> {
        if span.source != self.id {
            return None;
        }
        self.text().ok()?.get(span.start..span.end)
    }
    /// One-based Unicode-scalar column, not terminal width or UTF-16 units.
    pub fn location(&self, offset: usize) -> Option<(usize, usize)> {
        let text = self.text().ok()?;
        if offset > text.len() || !text.is_char_boundary(offset) {
            return None;
        }
        let line = self
            .lines
            .partition_point(|&x| x <= offset)
            .saturating_sub(1);
        Some((
            line + 1,
            text.get(self.lines[line]..offset)?.chars().count() + 1,
        ))
    }
}
#[derive(Default, Debug)]
pub struct SourceManager {
    files: Vec<SourceFile>,
}
impl SourceManager {
    pub fn add(&mut self, name: impl Into<String>, bytes: impl Into<Arc<[u8]>>) -> SourceId {
        let bytes = bytes.into();
        let id = SourceId(self.files.len());
        let mut lines = vec![0];
        for (i, b) in bytes.iter().enumerate() {
            if *b == b'\n' {
                lines.push(i + 1);
            }
        }
        let data = match String::from_utf8(bytes.to_vec()) {
            Ok(s) => Ok(s),
            Err(e) => Err((bytes, e.utf8_error())),
        };
        self.files.push(SourceFile {
            id,
            name: name.into(),
            data,
            lines,
        });
        id
    }
    pub fn load(&mut self, path: &Path) -> io::Result<SourceId> {
        let mut bytes = Vec::new();
        let limit = crate::Limits::default().source_bytes;
        File::open(path)?
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "source exceeds default frontend byte limit",
            ));
        }
        Ok(self.add(path.to_string_lossy(), bytes))
    }
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        self.files.get(id.0)
    }
}
