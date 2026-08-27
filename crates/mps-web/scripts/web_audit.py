#!/usr/bin/env python3
"""web_audit.py — mps-web documentation guards.

Run from the workspace root:
    python3 crates/mps-web/scripts/web_audit.py

Checks (all pure stdlib, portable across CI runners):
  1. i18n key parity — every key in en.ftl must exist in zh-CN.ftl and vice
     versa (no missing-key fallback, no orphan key).
  2. anchor integrity — every `#sec-*` href referenced in the page sources must
     resolve to a real `id: "sec-*"` defined in some page.
  3. stat-num hygiene — every `class: "stat-num"` must render a metrics::*
     constant (not a hardcoded numeric literal), so the doc counts never drift
     from the real source.

Exit code 0 = clean, 1 = at least one guard failed.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]  # workspace root
I18N = ROOT / "crates" / "mps-web" / "src" / "i18n" / "locales"
PAGES = ROOT / "crates" / "mps-web" / "src" / "pages"


def load_keys(path: Path) -> set[str]:
    keys: set[str] = set()
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.rstrip("\n")
        if not line or line.lstrip().startswith("#"):
            continue
        m = re.match(r"^([A-Za-z0-9_-]+)\s*=", line)
        if m:
            keys.add(m.group(1))
    return keys


def collect_sources() -> list[str]:
    return [p.read_text(encoding="utf-8") for p in sorted(PAGES.glob("*.rs"))]


def check_i18n_parity() -> list[str]:
    errors: list[str] = []
    en = load_keys(I18N / "en.ftl")
    zh = load_keys(I18N / "zh-CN.ftl")
    missing_zh = sorted(en - zh)
    orphan_en = sorted(zh - en)
    if missing_zh:
        errors.append(
            "i18n: keys present in en.ftl but MISSING in zh-CN.ftl: "
            + ", ".join(missing_zh)
        )
    if orphan_en:
        errors.append(
            "i18n: keys present in zh-CN.ftl but MISSING in en.ftl (orphans): "
            + ", ".join(orphan_en)
        )
    if not errors:
        print(f"[ok] i18n key parity: en={len(en)} zh={len(zh)} symmetric")
    return errors


def check_anchors() -> list[str]:
    errors: list[str] = []
    sources = collect_sources()
    defined = set()
    for src in sources:
        for m in re.finditer(r'id:\s*"sec-([a-z0-9-]+)"', src):
            defined.add("sec-" + m.group(1))
    referenced: set[str] = set()
    for src in sources:
        for m in re.finditer(r'#sec-([a-z0-9-]+)', src):
            referenced.add("sec-" + m.group(1))
    missing = sorted(referenced - defined)
    if missing:
        errors.append(
            "anchors: referenced #sec-* with no matching id: " + ", ".join(missing)
        )
    if not errors:
        print(
            f"[ok] anchor integrity: {len(referenced)} referenced, "
            f"{len(defined)} defined, all resolve"
        )
    return errors


def check_stat_num() -> list[str]:
    errors: list[str] = []
    for src in collect_sources():
        for m in re.finditer(
            r'class:\s*"stat-num"\s*,\s*"([^"]*)"', src
        ):
            literal = m.group(1)
            if literal.strip().isdigit():
                errors.append(
                    f"stat-num: hardcoded literal '{literal}' found; "
                    "use a metrics::* constant instead"
                )
    if not errors:
        print("[ok] stat-num hygiene: no hardcoded literals (all live metrics)")
    return errors


def main() -> int:
    print("=== mps-web documentation audit ===")
    all_errors: list[str] = []
    all_errors += check_i18n_parity()
    all_errors += check_anchors()
    all_errors += check_stat_num()
    if all_errors:
        print("\nFAILED:")
        for e in all_errors:
            print("  - " + e)
        return 1
    print("\nALL GUARDS PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
