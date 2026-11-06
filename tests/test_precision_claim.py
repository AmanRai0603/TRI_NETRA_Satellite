"""The flight software's documents hold to its build on precision (DR-fsw-002, F10): a document that claims a single-precision
build fails this test while `gcc -DADCS_REAL_FLOAT -fsyntax-only` does not build clean. The build is double only, the documents
say so, and the header refuses ADCS_REAL_FLOAT with an error rather than offering a build that warns.
Copyright (c) 2026 Agastya. All rights reserved."""
import re
import shutil
import subprocess
import unittest

from _path import ROOT

DOCS = ["fsw/pseudocode/00_conventions.md", "fsw/include/adcs_math.h"]
# a sentence that says a single-precision build exists, is supported or is tested (a sentence that says there is none does not)
CLAIM = re.compile(r"single[- ]precision build\b[^.]*?\b(exists|is tested|tested|supported|is supported)\b", re.I)
NEGATED = re.compile(r"\b(no|not|without|refused|waits)\b", re.I)


def claims_float_build(text):
    """Whether the text claims a single-precision build: a claiming sentence that does not deny it."""
    flat = re.sub(r"\s+", " ", text)
    return any(CLAIM.search(s) and not NEGATED.search(s) for s in re.split(r"(?<=[.;])\s", flat))


def verdict(claimed, builds_clean):
    """The test's rule: a claimed float build must build clean."""
    return (not claimed) or builds_clean


def float_build_warns():
    """gcc -DADCS_REAL_FLOAT -fsyntax-only on the flight software's shell: (builds clean, what it says)."""
    r = subprocess.run(["gcc", "-DADCS_REAL_FLOAT", "-Wall", "-Wextra", "-std=c99", "-fsyntax-only", "-I", str(ROOT / "fsw" / "include"),
                        "-I", str(ROOT / "fsw" / "alg" / "include"), "-I", str(ROOT / "fsw" / "src"), str(ROOT / "fsw" / "src" / "adcs_fsw.c")],
                       capture_output=True, text=True)
    out = (r.stdout + r.stderr).strip()
    return r.returncode == 0 and not out, out


class Claim(unittest.TestCase):
    def test_the_detector_finds_the_old_claim_and_not_the_new_words(self):
        old = ("- `real` is IEEE-754 double by default. A single-precision build (`ADCS_REAL_FLOAT`, Rust feature\n"
               "  `f32`) exists for FPU-single targets and is tested separately.\n")
        self.assertTrue(claims_float_build(old))
        self.assertTrue(claims_float_build("The single-precision build is supported on the M4F."))
        self.assertFalse(claims_float_build("There is no single-precision build: the build is double only."))
        self.assertFalse(claims_float_build("A single-precision build waits for the board's FPU."))
        self.assertFalse(claims_float_build("ADCS_REAL_FLOAT is refused at compile time."))

    def test_the_rule_fails_a_claim_the_build_does_not_keep(self):
        self.assertFalse(verdict(True, False))
        self.assertTrue(verdict(True, True))
        self.assertTrue(verdict(False, False))

    def test_no_document_claims_a_float_build_the_build_does_not_keep(self):
        claimed = [d for d in DOCS if claims_float_build((ROOT / d).read_text())]
        if not claimed:
            return
        if shutil.which("gcc") is None:
            self.skipTest("no gcc: cannot tell whether the float build is clean")
        clean, out = float_build_warns()
        self.assertTrue(verdict(True, clean), f"{', '.join(claimed)} claim a single-precision build; gcc -DADCS_REAL_FLOAT -fsyntax-only says:\n{out[:600]}")

    def test_the_documents_say_double_only_and_the_header_refuses_float(self):
        conv = (ROOT / "fsw/pseudocode/00_conventions.md").read_text()
        self.assertIn("double only", conv)
        self.assertIn("DR-fsw-002", conv)
        hdr = (ROOT / "fsw/include/adcs_math.h").read_text()
        self.assertRegex(hdr, r"#ifdef ADCS_REAL_FLOAT\s*\n#error")
        self.assertNotIn("typedef float adcs_real", hdr)

    def test_defining_the_float_build_is_refused_by_name(self):
        if shutil.which("gcc") is None:
            self.skipTest("no gcc")
        clean, out = float_build_warns()
        self.assertFalse(clean)
        self.assertIn("DR-fsw-002", out)


if __name__ == "__main__":
    unittest.main()
