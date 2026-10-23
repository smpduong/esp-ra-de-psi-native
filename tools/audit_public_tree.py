#!/usr/bin/env python3
"""Small stdlib-only exact-tree and content audit for the public checkpoint."""
from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

EXPECTED = frozenset("""Cargo.toml
Cargo.lock
README.md
public-files.txt
.gitignore
crates/recovery-core/Cargo.toml
crates/recovery-core/src/lib.rs
crates/recovery-core/tests/input_contract.rs
crates/native-host/Cargo.toml
crates/native-host/src/main.rs
docs/PUBLICATION_BOUNDARY.md
docs/STATUS.md
tools/audit_public_tree.py""".splitlines())
MAX_FILE = 5 * 1024 * 1024
MAX_TREE = 12 * 1024 * 1024
BAD_TEXT = [
    re.compile(p, re.I)
    for p in (
        "/" + r"Users/[^\s\"']+",
        r"/private/(?:tmp|var)/[^\s\"']+",
        r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----",
        r"\b(?:" + "AKIA" + r"|ASIA)[A-Z0-9]{16}\b",
        r"\bgh[pousr]_[A-Za-z0-9_]{20,}\b",
        r"\bgithub_pat_[A-Za-z0-9_]{20,}\b",
        r"include_bytes!\s*!?\s*\(",
        "\\." + "gitmodules|\\." + "gitattributes|\\." + "github" + "/workflows",
        r"(?:^|[/\\])[^\s]+\.(?:xci|nsp|nso|nro|rom|elf|bin|sav|key|tik|cert)(?:$|[?#\s])",
        r"(?:[A-Za-z]:\\" + r"Users\\|/" + r"home/)[^\s\"']+",
    )
]


def files_under(root: Path) -> tuple[set[str], list[str]]:
    found: set[str] = set()
    errors: list[str] = []
    for current, dirs, files in os.walk(root, followlinks=False):
        base = Path(current)
        dirs[:] = [d for d in dirs if d not in {".git", "target"}]
        for name in [*dirs, *files]:
            path = base / name
            rel = path.relative_to(root).as_posix()
            if path.is_symlink():
                errors.append(f"symlink forbidden: {rel}")
            elif path.is_file():
                found.add(rel)
    return found, errors


def audit(root: Path) -> list[str]:
    errors: list[str] = []
    if root.is_symlink() or not root.is_dir():
        return [f"not a real directory: {root}"]
    found, traversal_errors = files_under(root)
    errors.extend(traversal_errors)
    if found != EXPECTED:
        errors.append(f"tree differs from compiled exact allowlist; missing={sorted(EXPECTED-found)}, extra={sorted(found-EXPECTED)}")
    try:
        lines = (root / "public-files.txt").read_text(encoding="utf-8").splitlines()
        if len(lines) != len(set(lines)) or set(lines) != EXPECTED:
            errors.append("public-files.txt does not exactly match the compiled allowlist")
    except (OSError, UnicodeError):
        errors.append("public-files.txt is unreadable UTF-8")

    total = 0
    for rel in sorted(found & EXPECTED):
        path = root / rel
        try:
            size = path.stat().st_size
            total += size
            if size > MAX_FILE:
                errors.append(f"file too large: {rel} ({size} bytes)")
            text = path.read_bytes().decode("utf-8")
            for pattern in BAD_TEXT:
                if pattern.search(text):
                    errors.append(f"sensitive or disallowed pattern {pattern.pattern!r} in {rel}")
            if rel.endswith(".rs") and re.search(r"(?m)^\s*(?:global_asm!|asm!)", text):
                errors.append(f"assembly macro found in {rel}")
        except UnicodeError:
            errors.append(f"non-UTF-8 file: {rel}")
        except OSError as exc:
            errors.append(f"cannot read {rel}: {exc}")
    if total > MAX_TREE:
        errors.append(f"tree too large: {total} bytes")

    if (root / ".git").exists():
        try:
            tracked = subprocess.run(
                ["git", "-C", str(root), "ls-files", "--cached", "-z"],
                check=True, capture_output=True,
            ).stdout.decode("utf-8").split("\0")
            for path in filter(None, tracked):
                if path not in EXPECTED:
                    errors.append(f"tracked path outside exact allowlist: {path}")
                if path == "target" or path.startswith("target/"):
                    errors.append("tracked target/ build output found")
            staged = subprocess.run(
                ["git", "-C", str(root), "ls-files", "--stage", "-z"],
                check=True, capture_output=True,
            ).stdout.decode("utf-8").split("\0")
            for entry in filter(None, staged):
                mode = entry.split(" ", 1)[0]
                if mode in {"120000", "160000"}:
                    errors.append(f"symlink or submodule in git index: {entry.split(chr(9), 1)[-1]}")
        except (OSError, subprocess.CalledProcessError, UnicodeError) as exc:
            errors.append(f"cannot inspect git index: {exc}")
    return errors


def self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="public-tree-audit-") as temp:
        root = Path(temp)
        for rel in EXPECTED:
            path = root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("", encoding="utf-8")
        (root / "public-files.txt").write_text("".join(f"{p}\n" for p in sorted(EXPECTED)), encoding="utf-8")
        assert not audit(root), audit(root)
        (root / "README.md").write_text("AKIA" + "1234567890123456", encoding="utf-8")
        assert any("sensitive or disallowed" in e for e in audit(root)), audit(root)
        (root / "README.md").write_text("", encoding="utf-8")
        (root / "extra.txt").write_text("private", encoding="utf-8")
        assert any("extra" in e for e in audit(root)), audit(root)
        (root / "extra.txt").unlink()
        (root / "link").symlink_to(root / "README.md")
        assert any("symlink forbidden" in e for e in audit(root)), audit(root)
    print("self-test passed: valid fixture accepted; secret, extra file, and symlink rejected")


def main() -> int:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return 0
    if len(sys.argv) > 2:
        print("usage: python3 tools/audit_public_tree.py [--self-test]", file=sys.stderr)
        return 2
    root = Path(sys.argv[1]) if len(sys.argv) == 2 else Path(__file__).resolve().parents[1]
    errors = audit(root.absolute())
    if errors:
        print("PUBLIC TREE AUDIT FAILED", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"public tree audit passed: {len(EXPECTED)} exact allowlisted UTF-8 files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
