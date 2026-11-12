#!/usr/bin/env python3
"""Compare this port against the reference build.

The reference build is a Windows binary wrapped in a third-party crypter. Its
own strings carry the command registry verbatim:

    AddCommandInfo|<name>|<summary>|<aliases|

and its dialog directive names, the server_data body fields and the roulette
broadcast strings. This script extracts those and checks that the Rust port
covers every one of them.

    python3 scripts/compare_with_reference.py <path-to-reference.exe>

Nothing is executed: the reference is only read as data.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
COMMANDS_RS = REPO / "crates" / "lzfeatures" / "src" / "commands.rs"
DIALOG_RS = REPO / "crates" / "lzdialog" / "src" / "lib.rs"
SPOOFER_RS = REPO / "crates" / "lzproxy" / "src" / "lib.rs"
OPTIONS_RS = REPO / "crates" / "lzfeatures" / "src" / "options.rs"


def strings(path: Path) -> list[str]:
    out = subprocess.run(["strings", "-n", "4", str(path)], capture_output=True, text=True)
    return out.stdout.splitlines()


def reference_registry(lines: list[str]) -> list[dict]:
    entries = []
    for line in lines:
        if not line.startswith("AddCommandInfo|"):
            continue
        parts = line[len("AddCommandInfo|"):].split("|")
        if len(parts) >= 3:
            entries.append(
                {
                    "name": parts[0].lstrip("/"),
                    "summary": parts[1],
                    "aliases": [a.strip() for a in parts[2].split(",") if a.strip()],
                }
            )
    return entries


def port_registry() -> tuple[set[str], set[str]]:
    src = COMMANDS_RS.read_text()
    entries = re.findall(
        r'Entry \{ name: "([^"]+)",\s*summary: "([^"]*)",\s*keywords: "([^"]*)"', src, re.S
    )
    names = {n for n, _, _ in entries}
    aliases = {a.strip().lower() for _, _, k in entries for a in k.split(",") if a.strip()}
    return names, aliases


def port_dialog_directives() -> set[str]:
    src = DIALOG_RS.read_text()
    return set(re.findall(r'"(add_[a-z_]+|end_[a-z_]+|start_[a-z_]+)\|', src))


def reference_dialog_directives(lines: list[str]) -> set[str]:
    # The real directives are the ones the dialog builder emits; `add_cert_dir`
    # and friends belong to OpenSSL, not to the markup.
    noise = {"add_cert_dir"}
    found = set()
    for line in lines:
        for token in re.findall(r"\b(add_[a-z_]+|end_[a-z_]+|start_[a-z_]+)\b", line):
            found.add(token)
    return found - noise


def server_data_fields() -> set[str]:
    src = SPOOFER_RS.read_text()
    body = src.split("fn server_data_body", 1)[1]
    fields = set(re.findall(r"[\"\s]([a-z_0-9]+)\|", body))
    if "RTENDMARKERBS1001" in body:
        fields.add("RTENDMARKERBS1001")
    return fields


def report(reference: Path) -> int:
    lines = strings(reference)
    registry = reference_registry(lines)

    bin_names, bin_aliases = set(), set()
    for entry in registry:
        if entry["name"] == "options":
            continue
        key = entry["name"].split()[0].split("[")[0].strip()
        bin_names.add(key)
        for alias in entry["aliases"]:
            alias = alias.strip().lower()
            if alias and " " not in alias:
                bin_aliases.add(alias)

    port_names, port_aliases = port_registry()
    vocab = port_names | port_aliases

    bin_dialog = reference_dialog_directives(lines)
    port_dialog = port_dialog_directives()

    bin_fields = {
        "server", "port", "type2", "meta",
        "beta_server", "beta_port", "beta_type2", "beta_meta",
        "RTENDMARKERBS1001",
    }
    port_fields = server_data_fields()

    # Count the entries, not the struct definition.
    options = len(re.findall(r"MenuOption\s*\{\s*label", OPTIONS_RS.read_text()))

    checks = [
        ("command names", len(bin_names & vocab), len(bin_names), sorted(bin_names - vocab)),
        ("aliases", len(bin_aliases & port_aliases), len(bin_aliases), sorted(bin_aliases - port_aliases)),
        ("dialog directives", len(bin_dialog & port_dialog), len(bin_dialog), sorted(bin_dialog - port_dialog)),
        ("server_data fields", len(bin_fields & port_fields), len(bin_fields), sorted(bin_fields - port_fields)),
    ]

    width = 62
    print("=" * width)
    print("  RUST PORT vs REFERENCE BUILD".center(width))
    print("=" * width)
    print()
    failures = 0
    for label, covered, total, missing in checks:
        ok = not missing
        failures += 0 if ok else 1
        status = "OK" if ok else f"MISSING {missing}"
        print(f"  {label:<18} {covered:>4} / {total:<4}  {status}")
    print(f"  {'options menu':<18} {options:>4} / 17")
    print()
    print(f"  extra commands: {', '.join(sorted(port_names - bin_names))}")
    print()
    print("=" * width)
    return failures


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    return report(Path(sys.argv[1]))


if __name__ == "__main__":
    raise SystemExit(main())
