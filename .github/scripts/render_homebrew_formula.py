#!/usr/bin/env python3
"""Render the Homebrew formula for an art GitHub release."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


REPOSITORY = "awwwkshay-org/awesome-rust-templates"
TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
)


def read_checksum(checksums_dir: Path, version: str, target: str) -> str:
    archive = f"art-v{version}-{target}.tar.gz"
    checksum_file = checksums_dir / f"{archive}.sha256"
    try:
        checksum = checksum_file.read_text(encoding="utf-8").split()[0]
    except (FileNotFoundError, IndexError) as error:
        raise SystemExit(f"missing or empty checksum file: {checksum_file}") from error

    if not re.fullmatch(r"[0-9a-f]{64}", checksum):
        raise SystemExit(f"invalid SHA-256 in {checksum_file}: {checksum!r}")
    return checksum


def render_formula(version: str, checksums_dir: Path) -> str:
    if not re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", version):
        raise SystemExit(f"version must use MAJOR.MINOR.PATCH: {version!r}")

    checksums = {
        target: read_checksum(checksums_dir, version, target) for target in TARGETS
    }
    release_url = f"https://github.com/{REPOSITORY}/releases/download/v{version}"

    def source(target: str) -> str:
        archive = f"art-v{version}-{target}.tar.gz"
        return (
            f'      url "{release_url}/{archive}"\n'
            f'      sha256 "{checksums[target]}"'
        )

    return f'''class Art < Formula
  desc "Generate production-oriented Rust full-stack monorepos"
  homepage "https://github.com/{REPOSITORY}"
  version "{version}"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
{source("aarch64-apple-darwin")}
    else
{source("x86_64-apple-darwin")}
    end
  end

  on_linux do
    if Hardware::CPU.arm?
{source("aarch64-unknown-linux-gnu")}
    else
{source("x86_64-unknown-linux-gnu")}
    end
  end

  def install
    bin.install "art"
  end

  test do
    assert_match version.to_s, shell_output("#{{bin}}/art --version")
  end
end
'''


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--checksums-dir", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()

    formula = render_formula(args.version, args.checksums_dir)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula, encoding="utf-8")


if __name__ == "__main__":
    main()
