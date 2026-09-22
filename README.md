# fuzzyx

Fuzzy string matching in **Rust**: Levenshtein with an early-exit
bound, Jaro-Winkler, n-gram (Dice) similarity, ranked search — plus a
zero-dependency Python binding over the C ABI.

```text
$ fuzzyx similar aple apple apply banana maples
0.9467  apple
0.8889  maples
0.8267  apply
0.4722  banana

$ fuzzyx bench
levenshtein        40.8µs   avg top-1 score 0.796
jaro-winkler       44.7µs   avg top-1 score 0.927
bigram-dice        90.0µs   avg top-1 score 0.785
```

## The pieces

- **`levenshtein_bounded(a, b, max)`** — two-row DP with a proven early
  exit: as soon as the smallest value in the current row exceeds the
  bound, the final distance cannot come back under it and the call
  returns immediately. A length pre-filter rejects far-apart pairs in
  O(1). The bounded result is never wrong: whenever it reports a
  distance ≤ max, an exhaustive test asserts it equals the true one.
- **`jaro` / `jaro_winkler`** — windowed matching with transposition
  penalty and prefix boost; verified against published values
  (`martha`/`marhta` → 0.9611).
- **`ngram_similarity`** — Sørensen–Dice over character n-grams with
  multiset overlap (no double-counting repeated grams).
- **`ranked_search`** — stable top-k: ties keep candidate input order.
- **Python binding** — three `#[no_mangle] extern "C"` functions in the
  cdylib plus a 60-line ctypes wrapper (`python/fuzzyx.py`). No PyO3,
  no pip install, no build system: `cargo build --release` then
  `import fuzzyx` from the `python/` directory.

## CLI

```
fuzzyx distance <a> <b>
fuzzyx similar <query> <candidates...>
fuzzyx bench
```

## Development

```console
$ cargo test                          # 7 Rust tests incl. early-exit equivalence
$ cargo build --release               # also produces the cdylib
$ cd python && python3 -m unittest test_fuzzyx -v   # 4 binding tests
$ cargo clippy --all-targets -- -D warnings
```

## License

MIT — see [LICENSE](LICENSE).
