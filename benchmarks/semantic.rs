//! Reproducible provisional semantic baseline; not a cross-language comparison.
use cretes_frontend::{
    semantic::{analyze, ModuleInput, SemanticOptions},
    source::SourceManager,
};
use std::{hint::black_box, time::Instant};
fn main() {
    let mut source = String::new();
    for i in 0..500 {
        source.push_str(&format!("fn workload_{i}(x:i64)->Result[i64,text]{{let r:Result[i64,text]=Result::Ok(x+{i});return r;}}\n"));
    }
    let mut manager = SourceManager::default();
    let id = manager.add("semantic-baseline.cretes", source.as_bytes());
    let input = [ModuleInput {
        identity: "baseline",
        source: manager.get(id).unwrap(),
    }];
    for bits in [32, 64] {
        let mut samples = Vec::new();
        for _ in 0..7 {
            let start = Instant::now();
            let result = analyze(
                &input,
                SemanticOptions {
                    target_pointer_bits: bits,
                    ..SemanticOptions::default()
                },
            );
            samples.push(start.elapsed().as_micros());
            assert!(result.is_valid(), "{:?}", result.diagnostics);
            black_box(result);
        }
        samples.sort();
        println!(
            "target_bits={bits} bytes={} functions=500 samples=7 min_us={} median_us={} max_us={}",
            source.len(),
            samples[0],
            samples[3],
            samples[6]
        );
    }
}
