//! Entry selection is a caller decision; library analysis never invents a main.
use super::*;

/// Analyze an executable with an explicitly selected logical entry module.
/// Returns an input error when no source snapshots are supplied.
/// This validates the signature only; portable runtime exit codes are not evaluated.
pub fn analyze_entry(
    inputs: &[ModuleInput<'_>],
    options: SemanticOptions,
    entry: &str,
) -> Result<SemanticResult, &'static str> {
    if inputs.is_empty() {
        return Err("entry analysis requires at least one source snapshot");
    }
    let mut result = analyze(inputs, options);
    let selected = result.modules.iter().find(|m| m.identity == entry);
    let main = selected
        .and_then(|m| result.scopes[m.scope.0].bindings.get("main"))
        .copied();
    let valid = main
        .and_then(|id| result.signatures.get(&id))
        .is_some_and(|signature| {
            if !signature.parameters.is_empty() {
                return false;
            }
            let value = match result.types.get(signature.result) {
                Some(Type::Builtin { name, arguments }) if name == "Result" => {
                    arguments.first().copied()
                }
                _ => Some(signature.result),
            };
            value.is_some_and(|t| {
                matches!(
                    result.types.get(t),
                    Some(Type::Integer {
                        signed: true,
                        bits: 32
                    })
                )
            })
        });
    if !valid {
        // Missing modules still have an input source for an actionable location.
        let span = main
            .map(|id| result.symbols[id.0].span)
            .or_else(|| {
                selected.and_then(|m| m.ast.root.and_then(|n| m.ast.get(n)).map(|n| n.span))
            })
            .or_else(|| inputs.first().map(|i| i.source.span(0, 0)));
        if let Some(span) = span {
            let limit = options.limits.diagnostics.clamp(1, 1000);
            if result.diagnostics.len() < limit {
                result.diagnostics.push(Diagnostic::error(
                    "T001",
                    span,
                    "selected entry module requires main() -> i32 or main() -> Result[i32, E]",
                    "supply the executable's logical module identity; use analyze for libraries",
                ));
            }
        }
    }
    result
        .diagnostics
        .sort_by_key(|d| (d.primary.source, d.primary.start, d.code));
    Ok(result)
}
