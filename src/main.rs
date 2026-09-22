use std::process::exit;
use std::time::Instant;

use fuzzyx::{ranked_search, Metric};

const USAGE: &str = r#"fuzzyx v1.0.0 — fuzzy string matching library and benchmark

USAGE
  fuzzyx distance <a> <b>            Levenshtein distance
  fuzzyx similar <query> <c1> [c2..] ranked candidates (Jaro-Winkler)
  fuzzyx bench                       time all metrics on a word list
  fuzzyx (-h | --help | -V | --version)

Exit codes: 0 ok, 1 evaluation failure, 2 usage error."#;

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    match argv.first().map(|s| s.as_str()) {
        Some("-h" | "--help") => {
            println!("{USAGE}");
            return;
        }
        Some("-V" | "--version") => {
            println!("fuzzyx 1.0.0");
            return;
        }
        Some("distance") => {
            let (a, b) = match (argv.get(1), argv.get(2)) {
                (Some(a), Some(b)) => (a.clone(), b.clone()),
                _ => fail_usage("distance requires two words"),
            };
            println!("{}", fuzzyx::levenshtein(&a, &b));
        }
        Some("similar") => {
            let query = argv.get(1).cloned().unwrap_or_else(|| fail_usage("similar requires a query"));
            if argv.len() < 3 {
                fail_usage("similar requires at least one candidate");
            }
            let refs: Vec<&str> = argv[2..].iter().map(|s| s.as_str()).collect();
            for m in ranked_search(&query, &refs, 5, Metric::JaroWinkler) {
                println!("{:.4}  {}", m.score, m.text);
            }
        }
        Some("bench") => run_bench(),
        _ => fail_usage("missing or unknown command"),
    }
}

fn fail_usage(message: &str) -> ! {
    eprintln!("fuzzyx: {message}");
    eprintln!("fuzzyx: run 'fuzzyx --help' for usage");
    exit(2);
}

fn run_bench() {
    // a small dictionary and deliberately misspelled queries
    let dictionary: Vec<&str> = vec![
        "apple", "application", "apply", "banana", "bandana", "cherry", "chair",
        "kitten", "sitting", "kitchen", "hello", "hallo", "help", "world",
        "would", "wouldn't", "night", "nacht", "light", "sight",
    ];
    let queries: Vec<&str> = vec!["aple", "kitten", "helo", "nite", "bandanna", "wrld"];

    println!("fuzzyx benchmark: {} queries x {} candidates\n", queries.len(), dictionary.len());

    for metric in [Metric::Levenshtein, Metric::JaroWinkler, Metric::Bigram] {
        let name = match metric {
            Metric::Levenshtein => "levenshtein",
            Metric::JaroWinkler => "jaro-winkler",
            Metric::Bigram => "bigram-dice",
        };
        let start = Instant::now();
        let mut best_total = 0.0f64;
        for query in &queries {
            if let Some(top) = ranked_search(query, &dictionary, 1, metric).first() {
                best_total += top.score;
            }
        }
        let elapsed = start.elapsed();
        println!(
            "{:<14} {:>10.1?}   avg top-1 score {:.3}",
            name,
            elapsed,
            best_total / queries.len() as f64
        );
    }

    // early-exit demonstration: bounded vs unbounded on far-apart strings
    let a = "a".repeat(400);
    let b = format!("{}b", "a".repeat(399));
    let start = Instant::now();
    let _ = fuzzyx::levenshtein(&a, &b);
    let unbounded = start.elapsed();
    let start = Instant::now();
    let bounded_result = fuzzyx::levenshtein_bounded(&a, &b, 2);
    let bounded = start.elapsed();
    println!(
        "\nearly exit (400-char strings, bound 2): bounded={} in {:>8.1?}, unbounded in {:>8.1?}",
        bounded_result, bounded, unbounded
    );
}
