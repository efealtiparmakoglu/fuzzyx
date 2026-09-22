//! Fuzzy string matching metrics, written from scratch.
pub mod capi;

/// Levenshtein edit distance with two-row DP and a **max_distance early
/// exit**: once the smallest value in the current row exceeds
/// `max_distance`, the final distance cannot come back under it, so the
/// function returns `max_distance + 1` immediately. Length pre-filtering
/// rejects pairs whose length difference alone exceeds the bound.
pub fn levenshtein_bounded(a: &str, b: &str, max_distance: usize) -> usize {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let (a_len, b_len) = (a_bytes.len(), b_bytes.len());

    if a_len.abs_diff(b_len) > max_distance {
        return max_distance.saturating_add(1);
    }
    if a_len == 0 {
        return b_len.min(max_distance.saturating_add(1));
    }
    if b_len == 0 {
        return a_len.min(max_distance.saturating_add(1));
    }

    let mut previous: Vec<usize> = (0..=b_len).collect();
    let mut current = vec![0usize; b_len + 1];

    for (i, &ca) in a_bytes.iter().enumerate() {
        current[0] = i + 1;
        let mut row_min = current[0];
        for (j, &cb) in b_bytes.iter().enumerate() {
            let substitution = previous[j] + usize::from(ca != cb);
            let deletion = previous[j + 1] + 1;
            let insertion = current[j] + 1;
            let value = substitution.min(deletion).min(insertion);
            current[j + 1] = value;
            row_min = row_min.min(value);
        }
        if row_min > max_distance {
            return max_distance.saturating_add(1); // early exit: cannot recover
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b_len].min(max_distance.saturating_add(1))
}

/// Plain Levenshtein distance (no bound).
pub fn levenshtein(a: &str, b: &str) -> usize {
    levenshtein_bounded(a, b, usize::MAX)
}

/// Jaro similarity in [0, 1]: matches within a window, penalizing
/// transpositions.
pub fn jaro(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let (a_len, b_len) = (a_chars.len(), b_chars.len());
    if a_len == 0 && b_len == 0 {
        return 1.0;
    }
    if a_len == 0 || b_len == 0 {
        return 0.0;
    }
    let window = (a_len.max(b_len) / 2).saturating_sub(1);

    let mut a_matched = vec![false; a_len];
    let mut b_matched = vec![false; b_len];
    let mut matches = 0usize;

    for (i, &ca) in a_chars.iter().enumerate() {
        let start = i.saturating_sub(window);
        let end = (i + window + 1).min(b_len);
        for j in start..end {
            if !b_matched[j] && b_chars[j] == ca {
                a_matched[i] = true;
                b_matched[j] = true;
                matches += 1;
                break;
            }
        }
    }
    if matches == 0 {
        return 0.0;
    }

    // count transpositions: matched chars out of relative order
    let mut transpositions = 0usize;
    let mut j = 0usize;
    for i in 0..a_len {
        if !a_matched[i] {
            continue;
        }
        while !b_matched[j] {
            j += 1;
        }
        if a_chars[i] != b_chars[j] {
            transpositions += 1;
        }
        j += 1;
    }
    let m = matches as f64;
    (m / a_len as f64 + m / b_len as f64 + (m - transpositions as f64 / 2.0) / m) / 3.0
}

/// Jaro-Winkler: Jaro boosted for common prefixes.
pub fn jaro_winkler(a: &str, b: &str, prefix_scale: f64) -> f64 {
    let base = jaro(a, b);
    if base <= 0.0 {
        return base;
    }
    let prefix = a
        .chars()
        .zip(b.chars())
        .take(4)
        .take_while(|(x, y)| x == y)
        .count() as f64;
    let boost = prefix * prefix_scale * (1.0 - base);
    (base + boost).min(1.0)
}

/// Sørensen–Dice coefficient over character n-grams, in [0, 1].
pub fn ngram_similarity(a: &str, b: &str, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    let grams = |text: &str| -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        (0..chars.len().saturating_sub(n - 1))
            .map(|i| chars[i..i + n].iter().collect())
            .collect()
    };
    let (ga, gb) = (grams(a), grams(b));
    if ga.is_empty() || gb.is_empty() {
        return if a == b && ga.is_empty() && gb.is_empty() { 1.0 } else { 0.0 };
    }
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for gram in &ga {
        *counts.entry(gram.clone()).or_default() += 1;
    }
    let mut overlap = 0usize;
    for gram in &gb {
        if let Some(remaining) = counts.get_mut(gram) {
            if *remaining > 0 {
                *remaining -= 1;
                overlap += 1;
            }
        }
    }
    2.0 * overlap as f64 / (ga.len() + gb.len()) as f64
}

/// A candidate with its similarity score.
#[derive(Debug, Clone, PartialEq)]
pub struct Match {
    pub index: usize,
    pub text: String,
    pub score: f64,
}

/// The available metrics for ranked search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Levenshtein,
    JaroWinkler,
    Bigram,
}

impl Metric {
    fn score(self, query: &str, candidate: &str) -> f64 {
        match self {
            Metric::Levenshtein => {
                let distance = levenshtein(query, candidate);
                let max = query.chars().count().max(candidate.chars().count());
                if max == 0 { 1.0 } else { 1.0 - distance as f64 / max as f64 }
            }
            Metric::JaroWinkler => jaro_winkler(query, candidate, 0.1),
            Metric::Bigram => ngram_similarity(query, candidate, 2),
        }
    }
}

/// Ranked search: score every candidate, keep the top `limit` by score
/// (ties keep input order — the sort is stable).
pub fn ranked_search(
    query: &str,
    candidates: &[&str],
    limit: usize,
    metric: Metric,
) -> Vec<Match> {
    let mut matches: Vec<Match> = candidates
        .iter()
        .enumerate()
        .map(|(index, text)| Match {
            index,
            text: (*text).to_string(),
            score: metric.score(query, text),
        })
        .collect();
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(limit);
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_known_values() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("same", "same"), 0);
    }

    #[test]
    fn bounded_returns_cap_plus_one_when_too_far() {
        assert_eq!(levenshtein_bounded("kitten", "sitting", 2), 3);
        assert_eq!(levenshtein_bounded("kitten", "sitting", 3), 3);
        assert_eq!(levenshtein_bounded("abc", "xyz", 0), 1);
        assert_eq!(levenshtein_bounded("abcd", "ab", 10), 2);
    }

    #[test]
    fn early_exit_agrees_with_plain_distance() {
        // whenever bounded says "within k", the true distance must match
        let pairs = [("kitten", "sitting"), ("hello", "hallo"), ("aaaa", "aa"), ("x", "xylophone")];
        for (a, b) in pairs {
            for max in [0usize, 1, 2, 3, 10] {
                let bounded = levenshtein_bounded(a, b, max);
                let real = levenshtein(a, b);
                if bounded <= max {
                    assert_eq!(bounded, real, "{a} vs {b} max={max}");
                }
            }
        }
    }

    #[test]
    fn jaro_winkler_known_values() {
        let score = jaro_winkler("martha", "marhta", 0.1);
        assert!((score - 0.9611).abs() < 1e-3, "{score}");
        let score = jaro("dwaine", "duane");
        assert!((score - 0.8222).abs() < 1e-3, "{score}");
        assert_eq!(jaro("", "x"), 0.0);
        assert_eq!(jaro("", ""), 1.0);
    }

    #[test]
    fn bigram_similarity_ordering() {
        assert!(ngram_similarity("night", "nacht", 2) > ngram_similarity("night", "xyz", 2));
        assert_eq!(ngram_similarity("abc", "abc", 2), 1.0);
        assert_eq!(ngram_similarity("", "", 2), 1.0);
    }

    #[test]
    fn ranked_search_orders_and_limits() {
        let candidates = ["apple", "appl", "apply", "orange"];
        let top = ranked_search("apple", &candidates, 2, Metric::Levenshtein);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].text, "apple", "exact match first");
        assert_eq!(top[0].score, 1.0);
        // ties keep input order (stable): "appl" before "apply"
        let all = ranked_search("apple", &candidates, 4, Metric::JaroWinkler);
        let index_of = |text: &str| all.iter().position(|m| m.text == text).unwrap();
        assert!(index_of("appl") < index_of("apply") || all[index_of("appl")].score > all[index_of("apply")].score);
    }

    #[test]
    fn unicode_strings_are_handled() {
        assert_eq!(levenshtein("şeker", "şekerler"), 3); // +l, +e, +r
        let score = jaro_winkler("çığır", "çığır", 0.1);
        assert!((score - 1.0).abs() < 1e-9);
    }
}
