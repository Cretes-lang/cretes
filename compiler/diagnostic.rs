use crate::source::{SourceFile, Span};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    pub span: Span,
    pub message: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub primary: Span,
    pub secondary: Vec<Label>,
    pub notes: Vec<String>,
    pub help: String,
}
impl Diagnostic {
    pub fn error(
        code: &'static str,
        span: Span,
        message: impl Into<String>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            code,
            severity: Severity::Error,
            message: message.into(),
            primary: span,
            secondary: vec![],
            notes: vec![],
            help: help.into(),
        }
    }
    /// No raw source is printed: avoids terminal injection and accidental source disclosure.
    pub fn render(&self, source: &SourceFile) -> String {
        let pos = source
            .location(self.primary.start)
            .map(|(l, c)| format!("{l}:{c}"))
            .unwrap_or_else(|| format!("byte {}", self.primary.start));
        let mut out = format!(
            "{}:{}: {}: {}\n  help: {}",
            escaped(&source.name),
            pos,
            self.code,
            escaped(&self.message),
            escaped(&self.help)
        );
        for label in &self.secondary {
            let location = if label.span.source == source.id {
                source.location(label.span.start)
            } else {
                None
            };
            let location = location
                .map(|(l, c)| format!("{l}:{c}"))
                .unwrap_or_else(|| {
                    format!("source {} byte {}", label.span.source.0, label.span.start)
                });
            out.push_str(&format!("\n  at {location}: {}", escaped(&label.message)));
        }
        for note in &self.notes {
            out.push_str(&format!("\n  note: {}", escaped(note)));
        }
        out
    }
    pub fn json(&self) -> String {
        format!("{{\"schema_version\":1,\"offset_unit\":\"utf8-byte\",\"range\":\"half-open\",\"source_id\":{},\"start\":{},\"end\":{},\"code\":{},\"severity\":{},\"message\":{},\"help\":{},\"secondary\":[{}],\"notes\":[{}]}}", self.primary.source.0,self.primary.start,self.primary.end,quote(self.code),quote(match self.severity { Severity::Error=>"error",Severity::Warning=>"warning" }),quote(&self.message),quote(&self.help),self.secondary.iter().map(|l|format!("{{\"source_id\":{},\"start\":{},\"end\":{},\"message\":{}}}",l.span.source.0,l.span.start,l.span.end,quote(&l.message))).collect::<Vec<_>>().join(","),self.notes.iter().map(|s|quote(s)).collect::<Vec<_>>().join(","))
    }
}
fn escaped(s: &str) -> String {
    s.chars().take(512).flat_map(char::escape_default).collect()
}
pub fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
