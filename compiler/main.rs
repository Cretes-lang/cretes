use cretes_frontend::{lexer, parse_source, source::SourceManager, Limits};
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    process::ExitCode,
};
fn run() -> Result<bool, String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!("cretes-front (experimental, syntax only)\nUsage: cretes-front <lex|parse> <file.cretes> [--json-diagnostics]\nNot cretes check/build/run. No semantic analysis or execution.");
        return Ok(true);
    }
    if !(args.len() == 2 || args.len() == 3)
        || !(args[0] == "lex" || args[0] == "parse")
        || args.len() == 3 && args[2] != "--json-diagnostics"
    {
        return Err("usage: cretes-front <lex|parse> <file.cretes> [--json-diagnostics]".into());
    }
    let limits = Limits::default();
    let mut bytes = vec![];
    File::open(&args[1])
        .map_err(|e| e.to_string())?
        .take(limits.source_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    let mut sources = SourceManager::default();
    let id = sources.add(args[1].to_string_lossy(), bytes);
    let source = sources.get(id).ok_or("source snapshot unavailable")?;
    let result = if args[0] == "lex" {
        let l = lexer::lex(source, limits);
        cretes_frontend::FrontendResult {
            ast: Default::default(),
            tokens: l.tokens,
            trivia: l.trivia,
            diagnostics: l.diagnostics,
        }
    } else {
        parse_source(source, limits)
    };
    let mut err = io::stderr().lock();
    for d in &result.diagnostics {
        writeln!(
            err,
            "{}",
            if args.len() == 3 {
                d.json()
            } else {
                d.render(source)
            }
        )
        .map_err(|e| e.to_string())?;
    }
    let mut out = io::BufWriter::new(io::stdout().lock());
    if args[0] == "lex" {
        for t in &result.tokens {
            writeln!(
                out,
                "{:?} {}..{} {:?}",
                t.kind,
                t.span.start,
                t.span.end,
                source.slice(t.span).unwrap_or("")
            )
            .map_err(|e| e.to_string())?;
        }
    } else {
        write!(out, "{}", result.ast.dump()).map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())?;
    Ok(result.is_valid())
}
fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("cretes-front: {}", e.escape_default());
            ExitCode::from(2)
        }
    }
}
