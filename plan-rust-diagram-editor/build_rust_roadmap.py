"""Validate the roadmap and regenerate its offline dashboard, CSV backlog and Markdown status blocks.

roadmap.json is the only status source (ADR-003, D6). Run `python build_rust_roadmap.py` after
editing it; `python build_rust_roadmap.py --check` writes nothing and exits 1 when a block is stale.
"""

import csv
import json
import os
import posixpath
import re
import sys
import tempfile
from collections import Counter
from pathlib import Path
from urllib.parse import quote

ROOT = Path(__file__).resolve().parent


def validate(data):
    milestones = {item["id"]: item for item in data["milestones"]}
    tasks = {item["id"]: item for item in data["tasks"]}
    if len(milestones) != len(data["milestones"]) or len(tasks) != len(data["tasks"]):
        raise ValueError("Duplicate milestone or task ID")
    order = {key: index for index, key in enumerate(milestones)}
    visiting, visited = set(), set()

    def visit(task_id):
        if task_id in visiting:
            raise ValueError(f"Dependency cycle at {task_id}")
        if task_id in visited:
            return
        visiting.add(task_id)
        task = tasks[task_id]
        if task["milestone"] not in milestones:
            raise ValueError(f"Unknown milestone for {task_id}")
        status = task.get("status", data["default_task_status"])
        if status not in data["status_values"]:
            raise ValueError(f"Unknown status for {task_id}")
        if status == "done" and not task.get("evidence"):
            raise ValueError(f"Completed task {task_id} needs verification evidence")
        for field in ("title", "area", "deliverable", "acceptance"):
            if not task.get(field):
                raise ValueError(f"Missing {field} for {task_id}")
        for dependency in task["depends_on"]:
            if dependency not in tasks:
                raise ValueError(f"Unknown dependency {dependency} for {task_id}")
            if order[tasks[dependency]["milestone"]] > order[task["milestone"]]:
                raise ValueError(f"Later milestone dependency for {task_id}")
            visit(dependency)
            dependency_status = tasks[dependency].get("status", data["default_task_status"])
            if status == "done" and dependency_status != "done":
                raise ValueError(f"Completed task {task_id} has incomplete prerequisite")
        visiting.remove(task_id)
        visited.add(task_id)

    for task_id in tasks:
        visit(task_id)
    return milestones, tasks


PLAN_DIR = "plan-rust-diagram-editor"

# Markdown files whose status summaries are regenerated from roadmap.json, with their
# blocks in file order. Paths are relative to the repository root. Nothing else is written.
GENERATED_BLOCKS = {
    "README.md": ("summary",),
    f"{PLAN_DIR}/README.md": ("summary",),
    f"{PLAN_DIR}/RUST-ROADMAP.md": ("summary",),
    f"{PLAN_DIR}/05-ROADMAP.md": ("summary",),
    f"{PLAN_DIR}/PROGRESS-AND-NEXT-STEPS.md": ("summary", "milestones", "in-progress"),
    "app/docs/ui/README.md": ("ui-tasks",),
}

STATUS_ORDER = ("done", "in_progress", "blocked", "not_started")
STATUS_WORDS = {"done": "Done", "in_progress": "In progress", "blocked": "Blocked", "not_started": "Not started"}
BEGIN_RE = re.compile(r"<!-- roadmap:([a-z][a-z0-9-]*):begin\b[^>]*-->")
END_RE = re.compile(r"<!-- roadmap:([a-z][a-z0-9-]*):end -->")


def status_of(data, item):
    return item.get("status", data["default_task_status"])


def status_counts(data):
    """Validate the roadmap, then count tasks per status and milestone and find the current gate."""
    milestones, tasks = validate(data)
    statuses = list(dict.fromkeys((*STATUS_ORDER, *data["status_values"])))
    rows = {key: {"id": key, "title": item.get("title", ""), "total": 0, **{status: 0 for status in statuses}}
            for key, item in milestones.items()}
    for item in tasks.values():
        row = rows[item["milestone"]]
        row[status_of(data, item)] += 1
        row["total"] += 1
    order = list(milestones)
    release = next((index for index, key in enumerate(order) if milestones[key].get("release") == "v0.1"), None)
    gate = next((key for key in order if rows[key]["done"] < rows[key]["total"]), None)
    return {
        "total": len(tasks),
        "milestone_count": len(milestones),
        "by_status": {status: sum(row[status] for row in rows.values()) for status in data["status_values"]},
        "milestones": [rows[key] for key in order],
        "through_v01": None if release is None else sum(rows[key]["total"] for key in order[: release + 1]),
        "in_progress": [key for key, item in tasks.items() if status_of(data, item) == "in_progress"],
        "blocked": [key for key, item in tasks.items() if status_of(data, item) == "blocked"],
        "complete": [key for key in order if rows[key]["total"] and rows[key]["done"] == rows[key]["total"]],
        "gate": None if gate is None else {"id": gate, "done": rows[gate]["done"], "total": rows[gate]["total"]},
    }


# Escaping `[` and `]` also defeats image syntax `![...]`, so `!` stays as it is.
MARKDOWN_SYNTAX = re.compile(r"([\\`*_\[\]~|])")


def cell(value):
    """Show a roadmap value literally in Markdown text or a table cell.

    The result is one line; Markdown syntax (links, images, emphasis, code, `|`) is
    backslash-escaped and HTML (including roadmap markers) becomes entities.
    """
    text = MARKDOWN_SYNTAX.sub(r"\\\1", re.sub(r"\r\n|\r|\n", " ", str(value)))
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def link(path):
    return quote(path, safe="/")


def render_summary(data, roadmap_link):
    """One paragraph with every status count; `roadmap_link` is the relative path to roadmap.json."""
    counts = status_counts(data)
    by_status = counts["by_status"]
    updated = cell(data.get("last_updated") or data.get("date", "unknown"))
    scope = f"{counts['total']} tasks in {counts['milestone_count']} milestones"
    if counts["through_v01"] is None:
        scope += " (no milestone is marked v0.1)"
    else:
        scope += f", {counts['through_v01']} of them through v0.1"

    def listed(status, ids):
        count = f"{by_status.get(status, 0)} {STATUS_WORDS[status].lower()}"
        return count + (f" ({', '.join(map(cell, ids))})" if ids else "")

    parts = [f"{by_status.get('done', 0)} done", listed("in_progress", counts["in_progress"]),
             listed("blocked", counts["blocked"]), f"{by_status.get('not_started', 0)} not started"]
    complete = ", ".join(map(cell, counts["complete"])) or "none"
    gate = counts["gate"]
    gate_text = (f"{cell(gate['id'])}, {gate['done']} of {gate['total']} tasks done" if gate
                 else "none, every task is done")
    return (f"**Status** (generated from [roadmap.json]({link(roadmap_link)}), last updated {updated}): "
            f"{scope}. {', '.join(parts)}. Complete milestones: {complete}. Current gate: {gate_text}.")


def render_milestones(data):
    """A Markdown table of task counts per milestone with a totals row."""
    counts = status_counts(data)
    lines = ["| Milestone | Title | Done | In progress | Blocked | Not started | Total |",
             "|---|---|---|---|---|---|---|"]
    for row in counts["milestones"]:
        values = " | ".join(str(row[status]) for status in STATUS_ORDER)
        lines.append(f"| {cell(row['id'])} | {cell(row['title'])} | {values} | {row['total']} |")
    totals = " | ".join(str(counts["by_status"].get(status, 0)) for status in STATUS_ORDER)
    lines.append(f"| **All** | | {totals} | {counts['total']} |")
    return "\n".join(lines)


def render_in_progress(data):
    """A Markdown table of the tasks in progress or blocked, or a sentence when there are none."""
    validate(data)
    rows = [item for item in data["tasks"] if status_of(data, item) in ("in_progress", "blocked")]
    if not rows:
        return "No task is in progress or blocked."
    lines = ["| Task | Title | Milestone | Status |", "|---|---|---|---|"]
    for item in rows:
        lines.append(f"| {cell(item['id'])} | {cell(item['title'])} | {cell(item['milestone'])} "
                     f"| {STATUS_WORDS[status_of(data, item)]} |")
    return "\n".join(lines)


def evidence_path(item):
    """The file named by a task's first evidence entry (text before its first ':'), or None."""
    evidence = item.get("evidence")
    entries = evidence if isinstance(evidence, list) else [evidence]
    first = entries[0] if entries else None
    if not isinstance(first, str) or not first.strip():
        return None
    return first.split(":", 1)[0].strip() or None


def render_ui_tasks(data, from_dir):
    """A Markdown table of UI tasks; evidence links are relative to `from_dir` (repository-relative)."""
    validate(data)
    lines = ["| Task | What it delivers | Status |", "|---|---|---|"]
    for item in data["tasks"]:
        if not item["id"].startswith("UI-"):
            continue
        status = status_of(data, item)
        text = f"**{STATUS_WORDS[status]}**" if status == "done" else STATUS_WORDS[status]
        path = evidence_path(item)
        if path:
            text += f" — [evidence]({link(posixpath.relpath(path, from_dir or '.'))})"
        lines.append(f"| {cell(item['id'])} | {cell(item['title'])} | {text} |")
    return "\n".join(lines)


def fill_blocks(text, blocks):
    """Replace the content of each marked block in `text` (`\\n` newlines) with `blocks[name]`.

    Text outside the blocks is kept. Raise ValueError on any marker problem.
    """
    output, seen, current = [], set(), None
    for line in text.splitlines(keepends=True):
        if "<!-- roadmap:" not in line:
            if current is None:
                output.append(line)
            continue
        match = BEGIN_RE.search(line) or END_RE.search(line)
        if match is None:
            raise ValueError(f"Malformed roadmap marker: {line.strip()}")
        if line.strip() != match.group(0):
            raise ValueError(f"Roadmap marker must be on its own line: {line.strip()}")
        name = match.group(1)
        if match.re is BEGIN_RE:
            if current is not None:
                raise ValueError(f"Nested marker {name} inside block {current}")
            if name in seen:
                raise ValueError(f"Duplicate block {name}")
            if name not in blocks:
                raise ValueError(f"Unknown block {name}")
            current = name
            output.append(line if line.endswith("\n") else line + "\n")
            output.append(blocks[name].rstrip("\n") + "\n")
        else:
            if current is None:
                problem = "Duplicate end marker" if name in seen else "End marker"
                raise ValueError(f"{problem} for block {name} has no begin marker before it")
            if name != current:
                raise ValueError(f"Nested marker {name} inside block {current}")
            seen.add(name)
            current = None
            output.append(line)
    if current is not None:
        raise ValueError(f"Missing end marker for block {current}")
    missing = [name for name in blocks if name not in seen]
    if missing:
        raise ValueError(f"Missing begin marker for block {', '.join(missing)}")
    return "".join(output)


def read_lf(path):
    """Read a UTF-8 text file with its line endings normalized to `\\n`."""
    with open(path, encoding="utf-8", newline=None) as stream:
        return stream.read()


def generated_texts(data, repo):
    """The expected text (`\\n` newlines) of every file in GENERATED_BLOCKS, rendered in memory."""
    repo = Path(repo)
    result = {}
    for relative, names in GENERATED_BLOCKS.items():
        path = repo / relative
        if not path.resolve().is_relative_to(repo.resolve()):
            raise ValueError(f"{relative}: outside the repository")
        folder = posixpath.dirname(relative) or "."
        renderers = {
            "summary": lambda: render_summary(data, posixpath.relpath(f"{PLAN_DIR}/roadmap.json", folder)),
            "milestones": lambda: render_milestones(data),
            "in-progress": lambda: render_in_progress(data),
            "ui-tasks": lambda: render_ui_tasks(data, folder),
        }
        try:
            result[relative] = fill_blocks(read_lf(path), {name: renderers[name]() for name in names})
        except ValueError as error:
            raise ValueError(f"{relative}: {error}") from error
    return result


def write_keeping_newlines(path, text):
    """Replace `path` with `text` (`\\n` newlines) through a temporary file, keeping its line ending."""
    newline = "\r\n" if b"\r\n" in path.read_bytes() else "\n"
    handle, temporary = tempfile.mkstemp(dir=path.parent, prefix=f".{path.name}.", suffix=".tmp")
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(text.replace("\n", newline).encode("utf-8"))
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


HTML = r'''<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Diagram Studio · Rust roadmap</title>
<style>
:root{color-scheme:dark;--bg:#10161f;--panel:#192330;--line:#334359;--text:#e8eef6;--muted:#b3c0d0;--accent:#89e3c6}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--text);font:16px/1.55 system-ui,sans-serif}main{max-width:1280px;margin:auto;padding:36px 24px}h1{font-size:clamp(28px,4vw,44px);line-height:1.15;margin:12px 0}h2{font-size:23px}h3{font-size:17px;margin:8px 0}p{max-width:88ch}a{color:var(--accent)}.eyebrow,.muted{color:var(--muted)}.eyebrow{letter-spacing:.12em;text-transform:uppercase;font-size:12px}.notice{border-left:3px solid var(--accent);padding:12px 18px;background:var(--panel)}nav{display:flex;flex-wrap:wrap;gap:18px;margin:22px 0}.metrics{display:flex;gap:14px;flex-wrap:wrap;margin:26px 0}.metric{padding:14px 22px;border:1px solid var(--line);border-radius:12px;min-width:150px}.metric strong{display:block;font-size:30px;color:var(--accent)}.stages{display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:12px}.stage{background:var(--panel);padding:18px;border:1px solid var(--line);border-radius:12px}.stage p{font-size:14px}.stage button{font:inherit;text-align:left;color:var(--accent);background:none;border:0;cursor:pointer;padding:0}.stage button:hover{text-decoration:underline}.filters{display:flex;gap:12px;flex-wrap:wrap;align-items:end;margin:20px 0}.filters label{display:grid;gap:5px;font-size:14px}input,select,button.reset{font:inherit;background:var(--panel);color:var(--text);border:1px solid var(--line);border-radius:7px;padding:10px}input{min-width:min(320px,75vw)}button.reset{cursor:pointer}a:focus-visible,button:focus-visible,input:focus-visible,select:focus-visible,summary:focus-visible{outline:3px solid var(--accent);outline-offset:4px}.task{border:1px solid var(--line);border-radius:9px;margin:10px 0;background:var(--panel);scroll-margin-top:18px}summary{cursor:pointer;padding:16px}.task-body{padding:0 20px 18px}.task-body p{margin:10px 0}.tag{display:inline-block;font-size:12px;padding:2px 8px;margin:3px 6px 3px 0;border:1px solid var(--line);border-radius:20px;color:var(--muted)}.task-title{font-weight:600}.deps a{margin-right:12px}footer{margin-top:32px;font-size:14px;color:var(--muted)}code{font-size:.92em}noscript{display:block;padding:20px;background:var(--panel)}
</style></head><body><main>
<div class="eyebrow">Diagram Studio / independent Rust desktop editor</div>
<h1>Build the dependable editor first.</h1>
<p>Windows desktop, offline projects, reusable model objects and a simpler diagram workspace. Start with a complete class-modeling workflow; expand notation only after its foundation works.</p>
<p class="notice" id="state"></p>
<nav aria-label="Planning documents"><a href="PROGRESS-AND-NEXT-STEPS.md">Progress and next steps</a><a href="RUST-ROADMAP.md">Detailed roadmap</a><a href="RUST-ENGINEERING.md">Rust engineering rules</a><a href="rust-backlog.csv">CSV backlog</a><a href="UI-DESIGN.html">Final UI preview</a><a href="UI-DESIGN-AND-INTEGRATION.md">UI specification</a><a href="comparison/COMPARISON.html">Visual Paradigm comparison</a></nav>
<div class="metrics" id="metrics"></div>
<h2>Milestones, gated by evidence</h2><p class="muted">M0 → UX design → M1 → M2 → M3 (v0.1) → M4 → M5 → M6. These are dependency stages, not time estimates. M6 contains proposals requiring separate scope decisions.</p>
<section class="stages" id="stages" aria-label="Milestones"></section>
<h2 id="backlog">Implementation backlog</h2>
<div class="filters"><label>Search tasks<input id="search" type="search" placeholder="Try undo, SQLite, Arabic…"></label><label>Milestone<select id="milestone"><option value="">All milestones</option></select></label><label>Status<select id="status"><option value="">All statuses</option></select></label><button class="reset" id="reset" type="button">Reset filters</button></div>
<p id="count" class="muted" role="status" aria-live="polite"></p><section id="tasks" aria-label="Tasks"></section>
<noscript>Enable JavaScript to filter the embedded task data, or open the detailed Markdown roadmap and CSV backlog linked above.</noscript>
<footer>Generated from <a href="roadmap.json">roadmap.json</a>. Update that source and run <code>python build_rust_roadmap.py</code> to refresh this page and the CSV. Completion requires implementation and verification evidence. Task IDs are local tracking IDs.</footer>
</main><script id="roadmap-data" type="application/json">__DATA__</script><script>
'use strict';
const data=JSON.parse(document.getElementById('roadmap-data').textContent);
const $=id=>document.getElementById(id), statusOf=t=>t.status||data.default_task_status;
const label=s=>s.replaceAll('_',' '), firstRelease=new Set(['M0','UX','M1','M2','M3']);
const element=(tag,text,cls)=>{const e=document.createElement(tag);if(text!==undefined)e.textContent=text;if(cls)e.className=cls;return e;};
const done=data.tasks.filter(t=>statusOf(t)==='done').length;
$('state').textContent=`Application state: ${label(data.application_state)}. ${done} of ${data.tasks.length} tasks complete. Plan dated ${data.date}. This dashboard is a planning artifact; it is not the Rust application.`;
for(const [value,title] of [[data.milestones.length,'milestones'],[data.tasks.filter(t=>firstRelease.has(t.milestone)).length,'tasks through v0.1'],[data.tasks.filter(t=>!firstRelease.has(t.milestone)).length,'later tasks / proposals'],[done,'verified complete']]){const e=element('div',undefined,'metric');e.append(element('strong',String(value)),element('span',title));$('metrics').append(e);}
for(const m of data.milestones){const c=element('article',undefined,'stage'),b=element('button',`${m.id} · ${m.title}`);b.type='button';b.onclick=()=>{$('milestone').value=m.id;render();$('backlog').scrollIntoView();};c.append(b,element('p',`Target: ${m.release}`,'muted'),element('p',m.gate));$('stages').append(c);const o=element('option',`${m.id} · ${m.title}`);o.value=m.id;$('milestone').append(o);}
for(const s of data.status_values){const o=element('option',label(s));o.value=s;$('status').append(o);}
function paragraph(parent,title,value){const p=element('p');p.append(element('strong',title+' '),document.createTextNode(value));parent.append(p);}
function render(){const q=$('search').value.trim().toLowerCase(),m=$('milestone').value,s=$('status').value;
const visible=data.tasks.filter(t=>(!m||t.milestone===m)&&(!s||statusOf(t)===s)&&(!q||[t.id,t.title,t.area,t.deliverable,t.acceptance,...t.depends_on].join(' ').toLowerCase().includes(q)));
$('count').textContent=`Showing ${visible.length} of ${data.tasks.length} tasks`;$('tasks').replaceChildren();
for(const t of visible){const d=element('details',undefined,'task');d.id=t.id;const summary=element('summary');summary.append(element('span',`${t.id} · ${t.title}`,'task-title'),element('br'),element('span',t.milestone,'tag'),element('span',t.area,'tag'),element('span',label(statusOf(t)),'tag'));const body=element('div',undefined,'task-body');paragraph(body,'Deliverable:',t.deliverable);paragraph(body,'Acceptance:',t.acceptance);const deps=element('p',undefined,'deps');deps.append(element('strong','Prerequisites: '));if(!t.depends_on.length)deps.append(document.createTextNode('None — start here.'));for(const id of t.depends_on){const a=element('a',id);a.href='#'+id;a.onclick=e=>{e.preventDefault();$('search').value='';$('milestone').value='';$('status').value='';render();const target=$(id);target.open=true;target.scrollIntoView();history.replaceState(null,'','#'+id);};deps.append(a);}body.append(deps);if(t.evidence)paragraph(body,'Evidence:',Array.isArray(t.evidence)?t.evidence.join('; '):t.evidence);d.append(summary,body);$('tasks').append(d);}}
for(const id of ['search','milestone','status'])$(id).addEventListener('input',render);
$('reset').onclick=()=>{for(const id of ['search','milestone','status'])$(id).value='';render();};render();
const initial=$(location.hash.slice(1));if(initial&&initial.matches('details.task')){initial.open=true;initial.scrollIntoView();}
</script></body></html>
'''


def main(argv=None, root=ROOT):
    """Validate the roadmap, then regenerate the CSV, the HTML and the Markdown status blocks.

    With `--check`, write nothing and return 1 when a status block is stale. Return 2 on a
    usage, roadmap or marker error; nothing is written then either.
    """
    args = sys.argv[1:] if argv is None else list(argv)
    if any(arg != "--check" for arg in args):
        print("usage: python build_rust_roadmap.py [--check]", file=sys.stderr)
        return 2
    root = Path(root)
    repo = root.parent
    try:
        data = json.loads((root / "roadmap.json").read_text(encoding="utf-8"))
        milestones, tasks = validate(data)
        texts = generated_texts(data, repo)
        stale = [relative for relative, text in texts.items() if read_lf(repo / relative) != text]
    except (OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    if "--check" in args:
        if stale:
            print("Stale roadmap status blocks; run python build_rust_roadmap.py:")
            for relative in stale:
                print(f"  {relative}")
            return 1
        print(f"Roadmap status blocks are current in {len(texts)} files.")
        return 0
    fields = ["id", "milestone", "release", "title", "area", "status", "depends_on", "deliverable", "acceptance", "evidence"]
    with (root / "rust-backlog.csv").open("w", encoding="utf-8-sig", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=fields)
        writer.writeheader()
        for task in tasks.values():
            row = {key: task.get(key, "") for key in fields}
            row["release"] = milestones[task["milestone"]]["release"]
            row["status"] = task.get("status", data["default_task_status"])
            row["depends_on"] = "; ".join(task["depends_on"])
            if isinstance(row["evidence"], list):
                row["evidence"] = "; ".join(row["evidence"])
            writer.writerow(row)
    payload = json.dumps(data, ensure_ascii=False).replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")
    (root / "ROADMAP.html").write_text(HTML.replace("__DATA__", payload), encoding="utf-8")
    for relative in stale:
        write_keeping_newlines(repo / relative, texts[relative])
    counts = Counter(task["milestone"] for task in tasks.values())
    print(f"Validated {len(tasks)} tasks across {len(milestones)} milestones; dependency graph is acyclic.")
    print("Tasks by stage:", ", ".join(f"{key}={counts[key]}" for key in milestones))
    print("Generated ROADMAP.html and rust-backlog.csv")
    updated = f": {', '.join(stale)}" if stale else ""
    print(f"Updated status blocks in {len(stale)} of {len(texts)} files{updated}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
