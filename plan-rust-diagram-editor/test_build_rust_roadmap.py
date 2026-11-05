"""Tests for the roadmap validator. Run: python -m unittest discover -s plan-rust-diagram-editor"""

import copy
import json
import unittest
from pathlib import Path

from build_rust_roadmap import ROOT, validate


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
        self.assertIn("UI-12", prerequisites("R1-01"))
        self.assertTrue({"UI-%02d" % number for number in range(13, 18)} <= prerequisites("R2-08"))
        self.assertIn("UI-18", prerequisites("R3-07"))

    def test_committed_roadmap_is_valid(self):
        data = json.loads((Path(ROOT) / "roadmap.json").read_text(encoding="utf-8"))
        _, tasks = validate(data)
        self.assertEqual(tasks["R0-01"]["status"], "done")


if __name__ == "__main__":
    unittest.main()
