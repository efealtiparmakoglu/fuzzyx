"""fuzzyx — ctypes binding over the Rust cdylib.

Build first:  cargo build --release
The loader finds libfuzzyx.dylib/.so in ../target/release relative to
this file, or honors the FUZZYX_LIB environment variable.
"""

from __future__ import annotations

import ctypes
import os
from pathlib import Path


def _load() -> ctypes.CDLL:
    if env := os.environ.get("FUZZYX_LIB"):
        candidates = [Path(env)]
    else:
        base = Path(__file__).resolve().parent.parent / "target" / "release"
        candidates = [
            base / "libfuzzyx.dylib",
            base / "libfuzzyx.so",
            base / "fuzzyx.dll",
        ]
    for candidate in candidates:
        if candidate.exists():
            return ctypes.CDLL(str(candidate))
    raise FileNotFoundError(
        "fuzzyx native library not found; run 'cargo build --release' "
        f"(looked in: {', '.join(str(c) for c in candidates)})"
    )


_lib = _load()
_lib.fuzzyx_levenshtein.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
_lib.fuzzyx_levenshtein.restype = ctypes.c_size_t
_lib.fuzzyx_jaro_winkler.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
_lib.fuzzyx_jaro_winkler.restype = ctypes.c_double
_lib.fuzzyx_bigram.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
_lib.fuzzyx_bigram.restype = ctypes.c_double


def levenshtein(a: str, b: str) -> int:
    """Edit distance between two strings (Unicode-aware)."""
    return int(_lib.fuzzyx_levenshtein(a.encode("utf-8"), b.encode("utf-8")))


def jaro_winkler(a: str, b: str) -> float:
    """Jaro-Winkler similarity in [0, 1]."""
    return float(_lib.fuzzyx_jaro_winkler(a.encode("utf-8"), b.encode("utf-8")))


def bigram_similarity(a: str, b: str) -> float:
    """Sorensen-Dice coefficient over character bigrams, in [0, 1]."""
    return float(_lib.fuzzyx_bigram(a.encode("utf-8"), b.encode("utf-8")))
