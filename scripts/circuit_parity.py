#!/usr/bin/env python3
"""Spec 042 step 1: compare Rust-exported circuits with tracks/ JSON.

Usage:
    TDRACE_EXPORT_DIR=<dir> cargo test -p tdrace-app --test track_manager_tests \
        test_export_canonical_presets_to_git_repo -- --ignored
    python3 scripts/circuit_parity.py <dir> tracks

Prints one line per circuit: "same" or the list of fields that differ.
Removed with the Rust generators (spec 042 step 6).
"""

import json
import sys
from pathlib import Path

TOL = 1e-3


def diff(a, b, path, out):
    """Append to `out` the dotted paths where `a` and `b` differ."""
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                out.append(f"{path}.{k}" + ("(rust only)" if k in a else "(json only)"))
            else:
                diff(a[k], b[k], f"{path}.{k}", out)
    elif isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append(f"{path}[len {len(a)} vs {len(b)}]")
            return
        sub = []
        for x, y in zip(a, b):
            diff(x, y, path + "[]", sub)
            if sub:
                break
        out.extend(sub)
    elif (
        isinstance(a, (int, float))
        and isinstance(b, (int, float))
        and not isinstance(a, bool)
    ):
        if abs(a - b) > TOL * max(1.0, abs(a), abs(b)):
            out.append(f"{path}[{a} vs {b}]" if not path.endswith("[]") else path)
    elif a != b:
        out.append(
            f"{path}[{a!r} vs {b!r}]" if not isinstance(a, (dict, list)) else path
        )


def main():
    rust_dir, json_dir = Path(sys.argv[1]), Path(sys.argv[2])
    rust = {p.relative_to(rust_dir).as_posix() for p in rust_dir.glob("*/*.json")}
    disk = {p.relative_to(json_dir).as_posix() for p in json_dir.glob("*/*.json")}
    same = 0
    for rel in sorted(rust | disk):
        if rel not in disk:
            print(f"{rel}: rust only")
            continue
        if rel not in rust:
            print(f"{rel}: json only")
            continue
        out = []
        diff(
            json.loads((rust_dir / rel).read_text()),
            json.loads((json_dir / rel).read_text()),
            "",
            out,
        )
        if out:
            print(f"{rel}: " + "; ".join(out))
        else:
            same += 1
    print(f"# {same} identical of {len(rust | disk)}")


if __name__ == "__main__":
    main()
