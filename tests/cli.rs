use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Input(std::path::PathBuf);
impl Input {
    fn new(bytes: &[u8]) -> Self {
        let p = std::env::temp_dir().join(format!(
            "cretes-test-{}-{}.cretes",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&p, bytes).unwrap();
        Self(p)
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn run(mode: &str, input: &Input, json: bool) -> std::process::Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cretes-front"));
    c.arg(mode).arg(&input.0);
    if json {
        c.arg("--json-diagnostics");
    }
    c.output().unwrap()
}
#[test]
fn lex_golden() {
    let f = Input::new(b"let x = 42;");
    let o = run("lex", &f, false);
    assert!(o.status.success());
    assert!(o.stderr.is_empty());
    assert_eq!(String::from_utf8(o.stdout).unwrap(),"Let 0..3 \"let\"\nIdent 4..5 \"x\"\nEq 6..7 \"=\"\nInt 8..10 \"42\"\nSemi 10..11 \";\"\nEof 11..11 \"\"\n");
}
#[test]
fn parse_invalid_but_lex_valid() {
    let f = Input::new(b"fn");
    assert!(run("lex", &f, false).status.success());
    let o = run("parse", &f, true);
    assert_eq!(o.status.code(), Some(1));
    let d = String::from_utf8(o.stderr).unwrap();
    assert!(d.contains("\"code\":\"P001\""));
    assert!(d.contains("\"start\":2,\"end\":2"));
}
#[test]
fn invalid_utf8_exit() {
    let f = Input::new(&[0xff]);
    let o = run("parse", &f, true);
    assert_eq!(o.status.code(), Some(1));
    assert!(o.stdout.is_empty());
    assert!(String::from_utf8(o.stderr)
        .unwrap()
        .contains("\"code\":\"L001\""));
}
#[test]
fn missing_file_exit() {
    let f = Input::new(b"");
    fs::remove_file(&f.0).unwrap();
    assert_eq!(run("parse", &f, false).status.code(), Some(2));
}
#[test]
fn help_and_unknown_command() {
    assert!(Command::new(env!("CARGO_BIN_EXE_cretes-front"))
        .arg("--help")
        .output()
        .unwrap()
        .status
        .success());
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_cretes-front"))
            .arg("build")
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn valid_ast_output() {
    let f = Input::new(b"fn main() -> i32 { return 0; }");
    let a = run("parse", &f, false);
    let b = run("parse", &f, false);
    assert!(a.status.success());
    assert!(a.stderr.is_empty());
    assert_eq!(a.stdout, b.stdout);
    let s = String::from_utf8(a.stdout).unwrap();
    assert!(s.contains("Function {"));
    assert!(s.contains("Program {"));
}
#[test]
fn bounded_source_loading() {
    let f = Input::new(&vec![
        b' ';
        cretes_frontend::Limits::default().source_bytes + 1
    ]);
    let mut s = cretes_frontend::source::SourceManager::default();
    assert_eq!(
        s.load(&f.0).unwrap_err().kind(),
        std::io::ErrorKind::InvalidData
    );
    assert_eq!(run("lex", &f, true).status.code(), Some(1));
}
