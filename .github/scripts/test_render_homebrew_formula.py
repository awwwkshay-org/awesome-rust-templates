import importlib.util
import tempfile
import unittest
from pathlib import Path


MODULE_PATH = Path(__file__).with_name("render_homebrew_formula.py")
SPEC = importlib.util.spec_from_file_location("render_homebrew_formula", MODULE_PATH)
assert SPEC and SPEC.loader
FORMULA = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(FORMULA)


class RenderHomebrewFormulaTest(unittest.TestCase):
    def test_renders_every_release_target(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            checksums_dir = Path(directory)
            expected_checksums = {}
            for index, target in enumerate(FORMULA.TARGETS, start=1):
                checksum = f"{index:064x}"
                expected_checksums[target] = checksum
                filename = f"art-v1.2.3-{target}.tar.gz.sha256"
                (checksums_dir / filename).write_text(
                    f"{checksum}  art-v1.2.3-{target}.tar.gz\n", encoding="utf-8"
                )

            rendered = FORMULA.render_formula("1.2.3", checksums_dir)

            self.assertIn('version "1.2.3"', rendered)
            self.assertNotIn('depends_on "rust"', rendered)
            for target, checksum in expected_checksums.items():
                self.assertIn(f"art-v1.2.3-{target}.tar.gz", rendered)
                self.assertIn(f'sha256 "{checksum}"', rendered)

    def test_rejects_non_semantic_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(SystemExit):
                FORMULA.render_formula("latest", Path(directory))


if __name__ == "__main__":
    unittest.main()
