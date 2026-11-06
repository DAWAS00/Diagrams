"""Tests for the roadmap validator. Run: python -m unittest discover -s plan-rust-diagram-editor"""

import contextlib
import copy
import io
import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import build_rust_roadmap
from build_rust_roadmap import (
    GENERATED_BLOCKS,
    PLAN_DIR,
    ROOT,
    fill_blocks,
    generated_texts,
    main,
    read_lf,
    render_in_progress,
    render_milestones,
    render_summary,
    render_ui_tasks,
    status_counts,
    validate,
    write_keeping_newlines,
)


def task(task_id, milestone="M0", depends_on=(), **extra):
    item = {
        "id": task_id,
        "milestone": milestone,
        "title": f"Title {task_id}",
        "area": "Core",
        "deliverable": "Something",
        "acceptance": "It works",
        "depends_on": list(depends_on),
    }
    item.update(extra)
    return item


def roadmap(*tasks):
    return {
        "default_task_status": "not_started",
        "status_values": ["not_started", "in_progress", "done"],
        "milestones": [{"id": "M0", "release": "r0"}, {"id": "M1", "release": "r1"}],
        "tasks": list(tasks),
    }


class ValidateTests(unittest.TestCase):
    def assert_invalid(self, data, message):
        with self.assertRaisesRegex(ValueError, message):
            validate(data)

    def test_accepts_valid_roadmap_and_indexes_by_id(self):
        milestones, tasks = validate(roadmap(task("A"), task("B", "M1", ["A"])))
        self.assertEqual(list(milestones), ["M0", "M1"])
        self.assertEqual(list(tasks), ["A", "B"])

    def test_accepts_dependency_in_same_milestone(self):
        validate(roadmap(task("A"), task("B", "M0", ["A"])))

    def test_rejects_duplicate_task_id(self):
        self.assert_invalid(roadmap(task("A"), task("A")), "Duplicate")

    def test_rejects_duplicate_milestone_id(self):
        data = roadmap(task("A"))
        data["milestones"].append({"id": "M0", "release": "again"})
        self.assert_invalid(data, "Duplicate")

    def test_rejects_dependency_cycle(self):
        self.assert_invalid(roadmap(task("A", depends_on=["B"]), task("B", depends_on=["A"])), "cycle")

    def test_rejects_self_dependency(self):
        self.assert_invalid(roadmap(task("A", depends_on=["A"])), "cycle")

    def test_rejects_unknown_milestone(self):
        self.assert_invalid(roadmap(task("A", "M9")), "Unknown milestone for A")

    def test_rejects_unknown_status(self):
        self.assert_invalid(roadmap(task("A", status="blocked")), "Unknown status for A")

    def test_rejects_done_task_without_evidence(self):
        self.assert_invalid(roadmap(task("A", status="done")), "needs verification evidence")

    def test_accepts_done_task_with_evidence(self):
        validate(roadmap(task("A", status="done", evidence=["log"])))

    def test_rejects_each_missing_required_field(self):
        for field in ("title", "area", "deliverable", "acceptance"):
            with self.subTest(field=field):
                self.assert_invalid(roadmap(task("A", **{field: ""})), f"Missing {field} for A")

    def test_rejects_unknown_dependency(self):
        self.assert_invalid(roadmap(task("A", depends_on=["Z"])), "Unknown dependency Z for A")

    def test_rejects_dependency_on_later_milestone(self):
        self.assert_invalid(roadmap(task("A", depends_on=["B"]), task("B", "M1")), "Later milestone dependency for A")

    def test_rejects_done_task_with_incomplete_prerequisite(self):
        data = roadmap(task("A"), task("B", depends_on=["A"], status="done", evidence="log"))
        self.assert_invalid(data, "incomplete prerequisite")

    def test_does_not_mutate_input(self):
        data = roadmap(task("A"), task("B", "M1", ["A"]))
        before = copy.deepcopy(data)
        validate(data)
        self.assertEqual(data, before)

    def test_committed_ui_tasks_gate_the_right_milestones(self):
        data = json.loads((Path(ROOT) / "roadmap.json").read_text(encoding="utf-8"))
        _, tasks = validate(data)

        def prerequisites(task_id, seen=None):
            seen = set() if seen is None else seen
            for dependency in tasks[task_id]["depends_on"]:
                if dependency not in seen:
                    seen.add(dependency)
                    prerequisites(dependency, seen)
            return seen

        ui_tasks = [key for key in tasks if key.startswith("UI-")]
        self.assertEqual(len(ui_tasks), 18)
        self.assertEqual(len([key for key in ui_tasks if tasks[key]["milestone"] == "UX"]), 12)
        self.assertTrue({"UI-%02d" % number for number in range(1, 12)} <= prerequisites("UI-12"))
        # 2026-11-05 user decision: model work (R1) runs in parallel with the UI design,
        # but the class vertical slice still needs the native UI review and foundation gate.
        self.assertNotIn("UI-12", prerequisites("R1-01"))
        self.assertTrue({"UI-12", "R0-05"} <= prerequisites("R2-08"))
        self.assertTrue({"UI-%02d" % number for number in range(13, 18)} <= prerequisites("R2-08"))
        self.assertIn("UI-18", prerequisites("R3-07"))

    def test_committed_roadmap_is_valid(self):
        data = json.loads((Path(ROOT) / "roadmap.json").read_text(encoding="utf-8"))
        _, tasks = validate(data)
        self.assertEqual(tasks["R0-01"]["status"], "done")



# --- Generated status blocks (ADR-003 D6: roadmap.json is the only status source) ---


def status_roadmap():
    """M0 fully done, M1 (the v0.1 release) partial, then a later M2."""
    data = roadmap(
        task("A", status="done", evidence=["docs/a.md: proof"]),
        task("B", status="done", evidence="docs/b.md"),
        task("C", "M1", status="in_progress"),
        task("D", "M1", status="blocked"),
        task("E", "M1"),
        task("F", "M1"),
    )
    data["status_values"] = ["not_started", "in_progress", "blocked", "done"]
    data["last_updated"] = "2026-01-02"
    data["milestones"] = [
        {"id": "M0", "title": "Start", "release": "prototype"},
        {"id": "M1", "title": "Model", "release": "v0.1"},
        {"id": "M2", "title": "Later", "release": "later"},
    ]
    data["tasks"][-1]["milestone"] = "M2"
    return data


class StatusCountsTests(unittest.TestCase):
    def test_counts_each_status_with_default_applied(self):
        counts = status_counts(status_roadmap())
        self.assertEqual(counts["total"], 6)
        self.assertEqual(counts["by_status"], {"done": 2, "in_progress": 1, "blocked": 1, "not_started": 2})
        self.assertEqual(counts["in_progress"], ["C"])
        self.assertEqual(counts["blocked"], ["D"])

    def test_milestone_rows_sum_to_totals(self):
        counts = status_counts(status_roadmap())
        self.assertEqual([row["id"] for row in counts["milestones"]], ["M0", "M1", "M2"])
        for status, total in counts["by_status"].items():
            self.assertEqual(sum(row[status] for row in counts["milestones"]), total)
        self.assertEqual(sum(row["total"] for row in counts["milestones"]), counts["total"])

    def test_v01_count_stops_at_first_v01_milestone(self):
        self.assertEqual(status_counts(status_roadmap())["through_v01"], 5)

    def test_v01_count_stops_at_first_of_two_v01_milestones(self):
        data = status_roadmap()
        data["milestones"][2]["release"] = "v0.1"
        self.assertEqual(status_counts(data)["through_v01"], 5)

    def test_v01_count_is_none_without_a_v01_milestone(self):
        self.assertIsNone(status_counts(roadmap(task("A")))["through_v01"])

    def test_complete_milestones_and_current_gate(self):
        counts = status_counts(status_roadmap())
        self.assertEqual(counts["complete"], ["M0"])
        self.assertEqual(counts["gate"], {"id": "M1", "done": 0, "total": 3})

    def test_no_gate_when_every_task_is_done(self):
        counts = status_counts(roadmap(task("A", status="done", evidence="x")))
        self.assertIsNone(counts["gate"])
        self.assertEqual(counts["complete"], ["M0"])

    def test_rejects_invalid_roadmap(self):
        with self.assertRaisesRegex(ValueError, "needs verification evidence"):
            status_counts(roadmap(task("A", status="done")))


class RenderTests(unittest.TestCase):
    def test_summary_names_every_count(self):
        text = render_summary(status_roadmap(), "../plan/roadmap.json")
        for expected in ("last updated 2026-01-02", "6 tasks in 3 milestones", "5 of them through v0.1", "2 done",
                         "1 in progress (C)", "1 blocked (D)", "2 not started", "Complete milestones: M0.",
                         "Current gate: M1, 0 of 3 tasks done.", "](../plan/roadmap.json)"):
            self.assertIn(expected, text)

    def test_summary_says_none_when_nothing_is_blocked_or_complete(self):
        data = status_roadmap()
        data["tasks"] = [item for item in data["tasks"] if item["id"] not in ("A", "B", "D")]
        text = render_summary(data, "roadmap.json")
        self.assertIn("0 blocked,", text)
        self.assertIn("0 done", text)
        self.assertIn("Complete milestones: none.", text)

    def test_summary_when_every_task_is_done(self):
        text = render_summary(roadmap(task("A", status="done", evidence="x")), "roadmap.json")
        self.assertIn("Current gate: none, every task is done.", text)

    def test_summary_without_a_v01_milestone(self):
        self.assertIn("(no milestone is marked v0.1)", render_summary(roadmap(task("A")), "roadmap.json"))

    def test_milestone_table_has_a_row_per_milestone_and_a_total(self):
        lines = render_milestones(status_roadmap()).splitlines()
        self.assertEqual(lines[0], "| Milestone | Title | Done | In progress | Blocked | Not started | Total |")
        self.assertEqual(lines[2:], [
            "| M0 | Start | 2 | 0 | 0 | 0 | 2 |",
            "| M1 | Model | 0 | 1 | 1 | 1 | 3 |",
            "| M2 | Later | 0 | 0 | 0 | 1 | 1 |",
            "| **All** | | 2 | 1 | 1 | 2 | 6 |",
        ])

    def test_in_progress_table_lists_in_progress_and_blocked(self):
        lines = render_in_progress(status_roadmap()).splitlines()
        self.assertEqual(lines[0], "| Task | Title | Milestone | Status |")
        self.assertEqual(lines[2:], ["| C | Title C | M1 | In progress |", "| D | Title D | M1 | Blocked |"])

    def test_in_progress_says_none_when_empty(self):
        self.assertEqual(render_in_progress(roadmap(task("A"))), "No task is in progress or blocked.")

    def test_ui_table_rows_links_and_status_words(self):
        data = status_roadmap()
        data["tasks"] = [
            task("UI-01", status="done", evidence=["app/docs/ui/UI-01.md: tokens; more", "other.md"]),
            task("R1-01", status="done", evidence=["app/docs/r1.md"]),
            task("UI-02", status="in_progress", evidence="app/docs/ui/UI-02.md: a string, not a list"),
            task("UI-03", status="blocked"),
            task("UI-04"),
        ]
        lines = render_ui_tasks(data, "app/docs/ui").splitlines()
        self.assertEqual(lines[0], "| Task | What it delivers | Status |")
        self.assertEqual(lines[2:], [
            "| UI-01 | Title UI-01 | **Done** — [evidence](UI-01.md) |",
            "| UI-02 | Title UI-02 | In progress — [evidence](UI-02.md) |",
            "| UI-03 | Title UI-03 | Blocked |",
            "| UI-04 | Title UI-04 | Not started |",
        ])

    def test_markdown_syntax_in_titles_stays_literal(self):
        title = "![pixel](https://example.com/pixel) `code` *em* _u_ ~s~ \\ | <!-- roadmap:summary:end -->"
        data = roadmap(task("UI-01", title=title))
        row = render_ui_tasks(data, ".").splitlines()[2]
        self.assertEqual(
            row,
            "| UI-01 | !\\[pixel\\](https://example.com/pixel) \\`code\\` \\*em\\* \\_u\\_ \\~s\\~ \\\\ \\| "
            "&lt;!-- roadmap:summary:end --&gt; | Not started |",
        )
        block = render_ui_tasks(data, ".")
        filled = fill_blocks(marked("ui-tasks"), {"ui-tasks": block})
        self.assertEqual(fill_blocks(filled, {"ui-tasks": block}), filled)

    def test_ui_table_link_is_relative_and_percent_encoded(self):
        data = roadmap(task("UI-01", status="done", evidence="app/docs/ui/UI 01.md: x"))
        self.assertIn("[evidence](../app/docs/ui/UI%2001.md)", render_ui_tasks(data, "plan"))

    def test_hostile_title_stays_one_safe_cell(self):
        data = roadmap(task("UI-01", title="A | B\n<!-- roadmap:summary:end --> <b>&"))
        row = render_ui_tasks(data, ".").splitlines()[2]
        self.assertEqual(row, "| UI-01 | A \\| B &lt;!-- roadmap:summary:end --&gt; &lt;b&gt;&amp; | Not started |")
        filled = fill_blocks(marked("ui-tasks"), {"ui-tasks": render_ui_tasks(data, ".")})
        self.assertEqual(fill_blocks(filled, {"ui-tasks": render_ui_tasks(data, ".")}), filled)


BEGIN = "<!-- roadmap:{0}:begin (generated from roadmap.json by build_rust_roadmap.py; edit roadmap.json instead) -->"
END = "<!-- roadmap:{0}:end -->"


def marked(name, body="old"):
    return BEGIN.format(name) + "\n" + body + "\n" + END.format(name)


class FillBlocksTests(unittest.TestCase):
    def test_replaces_only_block_content(self):
        text = "before\n" + marked("summary") + "\nmiddle\n" + marked("ui-tasks", "x\ny") + "\nafter\n"
        result = fill_blocks(text, {"summary": "NEW", "ui-tasks": "T1\nT2"})
        expected = "before\n" + marked("summary", "NEW") + "\nmiddle\n" + marked("ui-tasks", "T1\nT2") + "\nafter\n"
        self.assertEqual(result, expected)

    def test_is_idempotent(self):
        once = fill_blocks("a\n" + marked("summary") + "\nb\n", {"summary": "NEW"})
        self.assertEqual(fill_blocks(once, {"summary": "NEW"}), once)

    def test_fills_an_empty_block(self):
        text = BEGIN.format("summary") + "\n" + END.format("summary") + "\n"
        self.assertEqual(fill_blocks(text, {"summary": "NEW"}), marked("summary", "NEW") + "\n")

    def test_errors(self):
        begin, end = BEGIN.format("summary"), END.format("summary")
        cases = {
            "missing begin": (end + "\n", "End marker for block summary"),
            "missing end": (begin + "\nold\n", "Missing end marker for block summary"),
            "end before begin": (end + "\n" + begin + "\n", "End marker for block summary"),
            "duplicate": (marked("summary") + "\n" + marked("summary") + "\n", "Duplicate block summary"),
            "unknown block": (marked("summary") + "\n" + marked("other") + "\n", "Unknown block other"),
            "nested": (begin + "\n" + marked("ui-tasks") + "\n" + end + "\n", "Nested marker ui-tasks"),
            "not in the file": ("x\n", "Missing begin marker for block summary"),
            "not alone on its line": ("see " + begin + "\nold\n" + end + "\n", "own line"),
            "malformed": ("<!-- roadmap:summary -->\n", "Malformed"),
        }
        for label, (text, message) in cases.items():
            with self.subTest(label):
                with self.assertRaisesRegex(ValueError, message):
                    fill_blocks(text, {"summary": "NEW"})


REPO = Path(ROOT).parent
STRAY_STATUS = re.compile(
    r"\d+ tasks? (remain|are) (not.started|complete)|tasks are complete|are all not started"
    r"|all new tasks are pending|\d+ of \d+ tasks done|remains? incomplete",
    re.IGNORECASE,
)
STALE_ORDER = re.compile(
    r"before M1|precedes M1|starts after UI-12|R1-01 now requires UI-12|before domain implementation|UX --> M1"
)
BLOCK = re.compile(r"<!-- roadmap:([a-z-]+):begin[^>]*-->.*?<!-- roadmap:\1:end -->", re.DOTALL)


def committed_roadmap():
    return json.loads((Path(ROOT) / "roadmap.json").read_text(encoding="utf-8"))


def stray_status(text):
    """Hand-written status text left outside the generated blocks, ignoring Markdown marks."""
    outside = BLOCK.sub("", text).replace("`", "").replace("*", "")
    return STRAY_STATUS.search(outside)


class CommittedBlocksTests(unittest.TestCase):
    def test_every_target_holds_exactly_its_blocks_in_order(self):
        for relative, names in GENERATED_BLOCKS.items():
            with self.subTest(relative):
                found = re.findall(r"<!-- roadmap:([a-z-]+):begin", read_lf(REPO / relative))
                self.assertEqual(found, list(names))

    def test_committed_summaries_match_roadmap(self):
        # Fails if any generated count or status in a committed file differs from roadmap.json.
        for relative, text in generated_texts(committed_roadmap(), REPO).items():
            with self.subTest(relative):
                self.assertEqual(read_lf(REPO / relative), text, "stale: run python build_rust_roadmap.py")

    def test_changing_one_status_changes_every_target(self):
        data = committed_roadmap()
        before = generated_texts(data, REPO)
        changed = copy.deepcopy(data)
        next(item for item in changed["tasks"] if item["id"] == "UI-07")["status"] = "in_progress"
        after = generated_texts(changed, REPO)
        for relative in GENERATED_BLOCKS:
            with self.subTest(relative):
                self.assertNotEqual(before[relative], after[relative])

    def test_no_stray_status_text_outside_blocks(self):
        for relative in GENERATED_BLOCKS:
            with self.subTest(relative):
                self.assertIsNone(stray_status(read_lf(REPO / relative)))

    def test_stray_status_check_flags_each_removed_sentence(self):
        for sentence in ("Three tasks are complete; R0-04 is in progress and 53 tasks are not started.",
                         "R0-01/R0-02/R0-03 are `done`; R0-04 is `in_progress`; 53 tasks remain `not_started`.",
                         "New UI tasks are all not started.", "All new tasks are pending.",
                         "**M0 remains incomplete: 3 of 5 tasks done.**", "36 tasks remain not started."):
            with self.subTest(sentence):
                self.assertIsNotNone(stray_status(sentence))

    def test_stray_status_check_ignores_text_inside_a_block(self):
        sentence = "53 tasks remain `not_started`"
        self.assertIsNotNone(stray_status("intro\n" + sentence + "\n" + marked("summary") + "\n"))
        self.assertIsNone(stray_status("intro\n" + marked("summary", sentence) + "\n"))

    def test_no_stale_ordering_guidance(self):
        for relative in GENERATED_BLOCKS:
            with self.subTest(relative):
                self.assertIsNone(STALE_ORDER.search(read_lf(REPO / relative)))

    def test_stale_ordering_check_flags_each_removed_phrase(self):
        for phrase in ("UI-12 review before M1", "UX design/native presentation now precedes M1",
                       "M1 — model, commands and storage (starts after UI-12)", "R1-01 now requires UI-12.",
                       "UX — full native presentation before domain implementation", "    UX --> M1[M1 Model]"):
            with self.subTest(phrase):
                self.assertIsNotNone(STALE_ORDER.search(phrase))


SCRIPT = Path(ROOT) / "build_rust_roadmap.py"


class RepositoryCopy:
    """A temporary copy of the roadmap, the generator, its outputs and every target file."""

    def __init__(self, tmp):
        self.repo = Path(tmp)
        self.plan = self.repo / PLAN_DIR
        self.plan.mkdir()
        for name in ("roadmap.json", "build_rust_roadmap.py", "ROADMAP.html", "rust-backlog.csv"):
            shutil.copy(Path(ROOT) / name, self.plan / name)
        for relative in GENERATED_BLOCKS:
            target = self.repo / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy(REPO / relative, target)

    def snapshot(self, only_markdown=False):
        files = sorted(path for path in self.repo.rglob("*") if path.is_file())
        if only_markdown:
            files = [path for path in files if path.suffix == ".md"]
        return {path: (path.read_bytes(), path.stat().st_mtime_ns) for path in files}

    def make_stale(self, relative):
        path = self.repo / relative
        text = path.read_bytes().decode("utf-8")
        changed = re.sub(r"\b14 done\b", "13 done", text) if "14 done" in text else text.replace("| UI-07 |", "| UI-07 (old) |")
        self.assertChanged(text, changed)
        path.write_bytes(changed.encode("utf-8"))

    @staticmethod
    def assertChanged(before, after):
        if before == after:
            raise AssertionError("fixture did not change the file")

    def run(self, *args):
        output, errors = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(errors):
            code = main(list(args), root=self.plan)
        return code, output.getvalue(), errors.getvalue()

    def run_script(self, *args):
        return subprocess.run([sys.executable, "-I", str(self.plan / "build_rust_roadmap.py"), *args],
                              capture_output=True, text=True, cwd=self.repo, check=False)


class CheckAndWriteTests(unittest.TestCase):
    def test_check_passes_on_current_files_and_writes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            before = copy_.snapshot()
            code, output, _ = copy_.run("--check")
            self.assertEqual(code, 0, output)
            self.assertEqual(copy_.snapshot(), before)

    def test_check_fails_on_a_stale_block_names_it_and_writes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            copy_.make_stale(f"{PLAN_DIR}/05-ROADMAP.md")
            before = copy_.snapshot()
            code, output, _ = copy_.run("--check")
            self.assertEqual(code, 1)
            self.assertIn("05-ROADMAP.md", output)
            self.assertEqual(copy_.snapshot(), before)

    def test_cli_exit_codes(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            before = copy_.snapshot()
            self.assertEqual(copy_.run_script("--check").returncode, 0)
            self.assertEqual(copy_.run_script("--bogus").returncode, 2)
            self.assertEqual(copy_.snapshot(), before)
            copy_.make_stale("app/docs/ui/README.md")
            before = copy_.snapshot()
            result = copy_.run_script("--check")
            self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
            self.assertIn("app/docs/ui/README.md", result.stdout)
            self.assertEqual(copy_.snapshot(), before)

    def test_marker_error_anywhere_writes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            first, last = list(GENERATED_BLOCKS)[0], list(GENERATED_BLOCKS)[-1]
            copy_.make_stale(first)
            path = copy_.repo / last
            path.write_bytes(re.sub(rb"<!-- roadmap:ui-tasks:end -->\r?\n", b"", path.read_bytes()))
            before = copy_.snapshot()
            code, _, errors = copy_.run()
            self.assertEqual(code, 2)
            self.assertIn(last, errors)
            self.assertIn("Missing end marker for block ui-tasks", errors)
            self.assertEqual(copy_.snapshot(), before)

    def test_regeneration_keeps_line_endings_and_a_second_run_writes_nothing(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            crlf, lf = copy_.repo / "app/docs/ui/README.md", copy_.repo / PLAN_DIR / "05-ROADMAP.md"
            for path, newline in ((crlf, "\r\n"), (lf, "\n")):
                path.write_bytes(read_lf(path).replace("\n", newline).encode("utf-8"))
            copy_.make_stale("app/docs/ui/README.md")
            copy_.make_stale(f"{PLAN_DIR}/05-ROADMAP.md")
            self.assertEqual(copy_.run()[0], 0)
            self.assertEqual(copy_.run("--check")[0], 0)
            raw_crlf, raw_lf = crlf.read_bytes(), lf.read_bytes()
            self.assertEqual(raw_crlf.count(b"\n"), raw_crlf.count(b"\r\n"))
            self.assertNotIn(b"\r", raw_lf)
            self.assertEqual(read_lf(crlf), read_lf(REPO / "app/docs/ui/README.md"))
            self.assertEqual(read_lf(lf), read_lf(REPO / PLAN_DIR / "05-ROADMAP.md"))
            before = copy_.snapshot(only_markdown=True)
            self.assertEqual(copy_.run()[0], 0)
            self.assertEqual(copy_.snapshot(only_markdown=True), before)
            self.assertEqual(sorted(p.name for p in copy_.repo.rglob("*.tmp")), [])

    def test_target_outside_the_repository_is_refused_and_nothing_is_written(self):
        with tempfile.TemporaryDirectory() as tmp:
            copy_ = RepositoryCopy(tmp)
            outside = Path(tmp).parent / "outside-roadmap-target.md"
            self.assertFalse(outside.exists())
            before = copy_.snapshot()
            targets = {**GENERATED_BLOCKS, "../outside-roadmap-target.md": ("summary",)}
            with mock.patch.dict(build_rust_roadmap.GENERATED_BLOCKS, targets, clear=True):
                code, _, errors = copy_.run()
            self.assertEqual(code, 2)
            self.assertIn("outside the repository", errors)
            self.assertFalse(outside.exists())
            self.assertEqual(copy_.snapshot(), before)

    def test_failed_replace_keeps_the_original_and_removes_the_temporary_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "target.md"
            path.write_bytes(b"old\r\ntext\r\n")
            with mock.patch.object(build_rust_roadmap.os, "replace", side_effect=OSError("disk full")):
                with self.assertRaisesRegex(OSError, "disk full"):
                    write_keeping_newlines(path, "new\n")
            self.assertEqual(path.read_bytes(), b"old\r\ntext\r\n")
            self.assertEqual([item.name for item in Path(tmp).iterdir()], ["target.md"])


if __name__ == "__main__":
    unittest.main()
