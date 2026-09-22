"""Python binding tests via ctypes; skipped when the cdylib is not built."""

import unittest
from pathlib import Path

try:
    import fuzzyx  # type: ignore[import-not-found]

    _LIB_AVAILABLE = True
except FileNotFoundError:
    fuzzyx = None  # type: ignore[assignment]
    _LIB_AVAILABLE = False


@unittest.skipUnless(_LIB_AVAILABLE, "cargo build --release output not found")
class TestBinding(unittest.TestCase):
    def test_levenshtein(self):
        self.assertEqual(fuzzyx.levenshtein("kitten", "sitting"), 3)
        self.assertEqual(fuzzyx.levenshtein("same", "same"), 0)
        self.assertEqual(fuzzyx.levenshtein("", "abc"), 3)

    def test_jaro_winkler(self):
        score = fuzzyx.jaro_winkler("martha", "marhta")
        self.assertAlmostEqual(score, 0.9611, delta=1e-3)
        self.assertEqual(fuzzyx.jaro_winkler("same", "same"), 1.0)

    def test_bigram(self):
        self.assertAlmostEqual(fuzzyx.bigram_similarity("abc", "abc"), 1.0)
        self.assertEqual(fuzzyx.bigram_similarity("aaa", "zzz"), 0.0)

    def test_unicode(self):
        self.assertEqual(fuzzyx.levenshtein("şeker", "şeker"), 0)
        self.assertGreater(fuzzyx.jaro_winkler("şeker", "şekerp"), 0.8)


if __name__ == "__main__":
    unittest.main()
