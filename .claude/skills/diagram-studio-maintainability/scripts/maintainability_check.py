#!/usr/bin/env python3
"""Maintainability check for the Diagram Studio Rust workspace.

Measures the structural rules from the diagram-studio-maintainability skill
so they do not depend on judgement. Run from the repository root:

    python .claude/skills/diagram-studio-maintainability/scripts/maintainability_check.py

FAIL  a rule is broken (the script exits with status 1)
WARN  structural debt to pay down before it grows

Options: --app <path to app/> (default: ./app), --json for machine output.
Standard library only; reads files, changes nothing.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

# Budgets. Raise them only with a recorded reason in the skill.
FILE_WARN_LINES = 500
FILE_SPLIT_LINES = 1000
STATE_PUB_FIELD_BUDGET = 12
HELPER_DUPLICATION_FILES = 3

# Model operations that only the coordinator (or the model itself) may call.
STATE_MUTATORS = (
    "select", "clear_selection", "toggle_in_selection", "select_relation",
    "open_diagram", "activate_tab", "close_tab", "reveal_class",
)
# Top-level functions allowed to apply state changes directly.
COORDINATORS = {
    "apply_intents", "run", "toggle_panel", "close_drawer", "close_showcase",
    "entry_screens",
}
# ShellState fields that drawing code may write directly: presentation and
# frame bookkeeping, not editor or document state. Every other field, including
# any new one, may change only in a coordinator, so adding a field forces a
# deliberate decision.
PRESENTATION_FIELDS = {
    "drafts", "navigator_search", "inspector_sections", "palette", "showcase",
    "showcase_return", "pending_focus", "text_had_focus", "applied_theme",
    "layout", "drawer", "drawer_return", "entry", "intents",
}
# Window sizes used by tests are not magic coordinates.
WINDOW_SIZES = {"800.0", "500.0", "1280.0", "1440.0", "900.0", "400.0", "300.0"}


def strip_comment(line: str) -> str:
    """Drop a // comment (good enough for this codebase's style)."""
    index = line.find("//")
    return line if index < 0 else line[:index]


def enclosing_items(lines: list[str]) -> list[str]:
    """For each line, the nearest preceding top-level item (`fn x` or `impl`)."""
    current = ""
    result = []
    item = re.compile(r"^(?:pub(?:\([^)]*\))? )?(?:fn (\w+)|impl\b)")
    for line in lines:
        match = item.match(line)
        if match:
            current = match.group(1) or "impl"
        result.append(current)
    return result


def check(app: Path) -> list[dict]:
    findings: list[dict] = []

    def add(level: str, rule: str, message: str, where: str = "") -> None:
        findings.append({"level": level, "rule": rule, "message": message, "where": where})

    sources = sorted(app.glob("crates/*/src/**/*.rs"))
    tests = sorted(app.glob("crates/*/tests/*.rs"))
    if not sources:
        add("FAIL", "layout", f"no Rust sources under {app}/crates/*/src")
        return findings

    for path in sources:
        rel = path.relative_to(app).as_posix()
        lines = path.read_text(encoding="utf-8").splitlines()
        code = [strip_comment(line) for line in lines]
        is_theme = rel.endswith("ui/theme.rs")
        items = enclosing_items(lines)
        source_text = "\n".join(lines)
        # Functions whose parameters include ShellState (the editor state).
        takes_state = {
            name
            for name, params in re.findall(r"^(?:pub(?:\([^)]*\))? )?fn (\w+)\s*(?:<[^>]*>)?\((.*?)\)", source_text, re.M | re.S)
            if "ShellState" in params
        }

        # 1. File size.
        if len(lines) > FILE_SPLIT_LINES:
            add("WARN", "file-size",
                f"{len(lines)} lines (> {FILE_SPLIT_LINES}): split by surface before adding features", rel)
        elif len(lines) > FILE_WARN_LINES:
            add("WARN", "file-size", f"{len(lines)} lines (> {FILE_WARN_LINES}): watch its growth", rel)

        for number, text in enumerate(code, start=1):
            where = f"{rel}:{number}"
            # 2. State changes outside the coordinator.
            for mutator in STATE_MUTATORS:
                if re.search(rf"\bstate\.{mutator}\(", text):
                    owner = items[number - 1]
                    if owner not in COORDINATORS and owner != "impl":
                        add("FAIL", "state-outside-coordinator",
                            f"`state.{mutator}()` in `{owner}`: queue a UiIntent instead", where)
            owner = items[number - 1]
            if owner in takes_state and owner not in COORDINATORS and owner != "show":
                write = re.search(
                    r"\bstate\.(\w+)(?:\.\w+)*\s*(?:[+\-*/]?=(?!=)|\.(?:push|clear|remove|insert|retain|truncate|extend)\()",
                    text,
                )
                if write and write.group(1) not in PRESENTATION_FIELDS:
                    add("FAIL", "state-outside-coordinator",
                        f"`state.{write.group(1)}` changed in `{owner}`: queue a UiIntent instead", where)
            # 3. Raw colours outside the theme (document colours are named constants).
            if not is_theme and re.search(r"Color32::from_rgba?\w*\(|\brgb\(0x", text):
                if not re.match(r"\s*(pub )?const DOCUMENT_\w+", text):
                    add("FAIL", "raw-colour", "colour literal outside ui/theme.rs: use a Palette token", where)
            # 4. Raw font sizes outside the theme.
            if not is_theme and re.search(r"FontId::(proportional|monospace)\(\s*\d", text):
                add("FAIL", "raw-font-size", "font size literal outside ui/theme.rs: use a TextRole", where)
            # 5. Forbidden focus test.
            if "egui_wants_keyboard_input" in text:
                add("FAIL", "focus-api",
                    "egui_wants_keyboard_input is true for any focus: use text_edit_focused", where)
            # 6. Panicking calls in production code.
            if re.search(r"\.(unwrap|expect)\(", text) and "#[cfg(test)]" not in "\n".join(lines):
                add("FAIL", "panic-in-production", "unwrap/expect in production code", where)

        # 7. Public fields on the shell state.
        source = "\n".join(lines)
        match = re.search(r"pub struct ShellState \{(.*?)\n\}", source, re.S)
        if match:
            public = len(re.findall(r"^\s+pub \w+:", match.group(1), re.M))
            if public > STATE_PUB_FIELD_BUDGET:
                add("WARN", "state-encapsulation",
                    f"ShellState has {public} public fields (budget {STATE_PUB_FIELD_BUDGET}): "
                    "expose read methods and change state through intents", rel)

    # 8. Test helpers re-defined in many files.
    helper_files: dict[str, set[str]] = defaultdict(set)
    for path in tests:
        for name in re.findall(r"^fn (\w+)\(", path.read_text(encoding="utf-8"), re.M):
            helper_files[name].add(path.name)
    for name, files in sorted(helper_files.items()):
        if len(files) >= HELPER_DUPLICATION_FILES:
            add("WARN", "duplicated-test-helper",
                f"`fn {name}` is defined in {len(files)} test files: move it to tests/common/mod.rs",
                ", ".join(sorted(files)))

    # 9. Magic screen coordinates in tests.
    for path in tests:
        rel = path.relative_to(app).as_posix()
        for number, text in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            text = strip_comment(text)
            if not re.search(r"\b(x0|x1|y0|y1)\b", text):
                continue
            for literal in re.findall(r"\b\d{3,}\.\d+\b", text):
                if literal not in WINDOW_SIZES:
                    add("WARN", "magic-coordinate",
                        f"coordinate literal {literal}: derive it from a named constant or another node's bounds",
                        f"{rel}:{number}")

    # 10. Pure logic functions no test calls directly. Drawing functions
    # (taking a Ui) are covered through the headless UI tests and the audit.
    test_text = "\n".join(path.read_text(encoding="utf-8") for path in tests)
    # Signatures may span several lines: read up to the opening brace.
    signature = re.compile(r"^pub fn (\w+)(.*?)\{", re.M | re.S)
    for path in sorted(app.glob("crates/diagram-desktop/src/ui/*.rs")):
        rel = path.relative_to(app).as_posix()
        for match in signature.finditer(path.read_text(encoding="utf-8")):
            name, rest = match.group(1), match.group(2)
            if "egui::Ui" in rest or "InputState" in rest or "egui::Context" in rest:
                continue
            if not re.search(rf"\b{name}\b", test_text):
                add("WARN", "untested-logic-fn", f"pure function `{name}` has no direct test", rel)

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--app", default="app", help="path to the Cargo workspace (default: app)")
    parser.add_argument("--json", action="store_true", help="print findings as JSON")
    args = parser.parse_args()

    findings = check(Path(args.app))
    failures = [f for f in findings if f["level"] == "FAIL"]
    if args.json:
        print(json.dumps({"failures": len(failures), "findings": findings}, indent=2))
    else:
        by_rule: dict[str, list[dict]] = defaultdict(list)
        for finding in findings:
            by_rule[f'{finding["level"]} {finding["rule"]}'].append(finding)
        for heading in sorted(by_rule, key=lambda h: (not h.startswith("FAIL"), h)):
            print(f"\n{heading} ({len(by_rule[heading])})")
            for finding in by_rule[heading]:
                print(f"  - {finding['where']}: {finding['message']}" if finding["where"] else f"  - {finding['message']}")
        warnings = len(findings) - len(failures)
        print(f"\nSummary: {len(failures)} FAIL, {warnings} WARN")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
