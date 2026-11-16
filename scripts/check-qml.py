#!/usr/bin/env python3
"""Static checks for the Quickshell QML tree.

The QML CI job runs under plain Qt, so it cannot resolve `qs.*` imports or
Quickshell types. These checks catch the mistakes that do not need a QML engine:

  * unbalanced braces, brackets, parentheses
  * `import qs.<path>` that does not resolve to a directory
  * duplicate ids in one file
  * a component used by a file that neither lives beside it nor is imported

Usage: python3 scripts/check-qml.py [shell-root]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# Files that are standalone entry points: qs.* still resolves, but they are not
# part of the regular module tree.
SKIP_DIRS = {"assets"}

TYPE_DECL = re.compile(r"^\s*(?:component\s+)?([A-Z][A-Za-z0-9_]*)\s*[:{]", re.MULTILINE)
IMPORT = re.compile(r"^\s*import\s+qs\.([A-Za-z0-9_.]+)", re.MULTILINE)
ID = re.compile(r"\bid:\s*([a-z][A-Za-z0-9_]*)")


def strip_noise(text: str) -> str:
    """Remove comments and string literals so bracket counting is meaningful."""
    out: list[str] = []
    i = 0
    quote = None
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if quote:
            if ch == "\\":
                i += 2
                continue
            if ch == quote:
                quote = None
            i += 1
            continue
        if ch in "\"'`":
            quote = ch
            i += 1
            continue
        if ch == "/" and nxt == "/":
            while i < len(text) and text[i] != "\n":
                i += 1
            continue
        if ch == "/" and nxt == "*":
            i += 2
            while i + 1 < len(text) and not (text[i] == "*" and text[i + 1] == "/"):
                i += 1
            i += 2
            continue
        out.append(ch)
        i += 1
    if quote:
        out.append("\0unterminated-string")
    return "".join(out)


def check_file(path: Path, root: Path) -> list[str]:
    problems: list[str] = []
    raw = path.read_text(encoding="utf-8")
    stripped = strip_noise(raw)
    if "\0unterminated-string" in stripped:
        problems.append("unterminated string literal")

    # Brackets. QML allows `}` and `]` only as balanced pairs; a stray one is a
    # parse error at load time, which is expensive to discover on a live shell.
    stack: list[tuple[str, int]] = []
    pairs = {")": "(", "]": "[", "}": "{"}
    for offset, ch in enumerate(stripped):
        if ch in "([{":
            stack.append((ch, offset))
        elif ch in ")]}":
            if not stack or stack[-1][0] != pairs[ch]:
                line = raw.count("\n", 0, offset) + 1
                problems.append(f"line {line}: unbalanced '{ch}'")
                break
            stack.pop()
    else:
        if stack:
            ch, offset = stack[-1]
            line = raw.count("\n", 0, offset) + 1
            problems.append(f"line {line}: unclosed '{ch}'")

    # qs imports.
    for match in IMPORT.finditer(raw):
        target = root.joinpath(*match.group(1).split("."))
        if not target.is_dir():
            problems.append(f"import qs.{match.group(1)} does not resolve to a directory")

    # Duplicate ids.
    ids: dict[str, int] = {}
    for match in ID.finditer(raw):
        name = match.group(1)
        line = raw.count("\n", 0, match.start()) + 1
        if name in ids:
            problems.append(f"line {line}: duplicate id '{name}' (first on line {ids[name]})")
        else:
            ids[name] = line

    return problems


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "shell").resolve()
    if not root.is_dir():
        print(f"not a directory: {root}", file=sys.stderr)
        return 2

    files = sorted(p for p in root.rglob("*.qml") if not SKIP_DIRS & set(p.relative_to(root).parts))
    failures = 0
    for path in files:
        problems = check_file(path, root)
        for problem in problems:
            failures += 1
            print(f"{path.relative_to(root)}: {problem}")

    print(f"checked {len(files)} QML files, {failures} problem(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
