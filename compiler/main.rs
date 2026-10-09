use cretes_frontend::{lexer, parse_source, source::SourceManager, Limits};
use std::{
    env,
    fs::File,
    io::{self, Read, Write},
    path::Path,
    process::ExitCode,
};

fn run() -> Result<bool, String> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.first().is_some_and(|a| a == "analyze") {
        return run_semantic(&args[1..]);
    }
    if args.len() == 1 && args[0] == "--help" {
        println!(
            "cretes-front (experimental candidate frontend)\n\
             Usage: cretes-front <lex|parse> <file.cretes> [--json-diagnostics]\n\
             Semantic inspection: cretes-front analyze --target-bits <32|64> --module <identity> <file> [--entry <identity>] [--json-diagnostics]\n\
             Not cretes check/build/run. Semantic success does not certify complete language validity or move/loan safety. No execution."
        );
        return Ok(true);
    }
    if !(args.len() == 2 || args.len() == 3)
        || !(args[0] == "lex" || args[0] == "parse")
        || (args.len() == 3 && args[2] != "--json-diagnostics")
    {
        return Err("usage: cretes-front <lex|parse> <file.cretes> [--json-diagnostics]".into());
    }

    let limits = Limits::default();
    let mut bytes = vec![];
    File::open(Path::new(&args[1]))
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

fn run_semantic(args: &[std::ffi::OsString]) -> Result<bool, String> {
    use cretes_frontend::semantic::{analyze, analyze_entry, ModuleInput, SemanticOptions};

    let mut options = SemanticOptions::default();
    let mut target = None;
    let mut entry = None;
    let mut json = false;
    let mut specifications = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].to_str() {
            Some("--target-bits") if target.is_none() && i + 1 < args.len() => {
                target = match args[i + 1].to_str() {
                    Some("32") => Some(32),
                    Some("64") => Some(64),
                    _ => return Err("target bits must be 32 or 64".into()),
                };
                i += 2;
            }
            Some("--entry") if entry.is_none() && i + 1 < args.len() => {
                entry = Some(
                    args[i + 1]
                        .to_str()
                        .ok_or("entry identity must be UTF-8")?
                        .to_owned(),
                );
                i += 2;
            }
            Some("--module") if i + 2 < args.len() => {
                if specifications.len() >= options.modules {
                    return Err("module limit exceeded".into());
                }
                specifications.push((
                    args[i + 1]
                        .to_str()
                        .ok_or("module identity must be UTF-8")?
                        .to_owned(),
                    args[i + 2].clone(),
                ));
                i += 3;
            }
            Some("--json-diagnostics") if !json => {
                json = true;
                i += 1;
            }
            _ => return Err("invalid analyze arguments; see --help".into()),
        }
    }

    options.target_pointer_bits = target.ok_or("analyze requires explicit --target-bits 32 or 64")?;
    if specifications.is_empty() {
        return Err("analyze requires at least one --module <identity> <file>".into());
    }

    let mut sources = SourceManager::default();
    let mut ids = Vec::new();
    let mut remaining_bytes = options.limits.source_bytes;

    for (identity, path) in &specifications {
        remaining_bytes = remaining_bytes
            .checked_sub(identity.len())
            .ok_or("semantic session input budget exceeded")?;

        let mut bytes = Vec::new();
        File::open(Path::new(path))
            .map_err(|e| e.to_string())?
            .take((options.limits.source_bytes.min(remaining_bytes)).saturating_add(1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;

        if bytes.len() > options.limits.source_bytes {
            return Err("source file byte limit exceeded".into());
        }

        remaining_bytes = remaining_bytes
            .checked_sub(bytes.len())
            .ok_or("semantic session input budget exceeded")?;

        ids.push(sources.add(path.to_string_lossy(), bytes));
    }

    let inputs: Vec<_> = specifications
        .iter()
        .zip(ids)
        .map(|((identity, _), id)| ModuleInput {
            identity,
            source: sources.get(id).expect("newly inserted source"),
        })
        .collect();

    let result = if let Some(entry) = entry {
        analyze_entry(&inputs, options, &entry).map_err(str::to_owned)?
    } else {
        analyze(&inputs, options)
    };

    let mut err = io::stderr().lock();
    for diagnostic in &result.diagnostics {
        let text = if json {
            diagnostic.json()
        } else if let Some(source) = sources.get(diagnostic.primary.source) {
            diagnostic.render(source)
        } else {
            diagnostic.message.clone()
        };
        writeln!(err, "{text}").map_err(|e| e.to_string())?;
    }

    let mut out = io::BufWriter::new(io::stdout().lock());
    write!(out, "{}", result.dump()).map_err(|e| e.to_string())?;
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
