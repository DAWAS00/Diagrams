"""Validate the roadmap and regenerate its offline dashboard and CSV backlog."""

import csv
import json
from collections import Counter
from pathlib import Path

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
<nav aria-label="Planning documents"><a href="RUST-ROADMAP.md">Detailed roadmap</a><a href="RUST-ENGINEERING.md">Rust engineering rules</a><a href="rust-backlog.csv">CSV backlog</a><a href="GUI-CONCEPT.html">Workspace concept</a><a href="comparison/COMPARISON.html">Visual Paradigm comparison</a></nav>
<div class="metrics" id="metrics"></div>
<h2>Seven milestones, gated by evidence</h2><p class="muted">M0 → M1 → M2 → M3 (v0.1) → M4 → M5 → M6. These are dependency stages, not time estimates. M6 contains proposals requiring separate scope decisions.</p>
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
const label=s=>s.replaceAll('_',' '), firstRelease=new Set(['M0','M1','M2','M3']);
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


def main():
    data = json.loads((ROOT / "roadmap.json").read_text(encoding="utf-8"))
    milestones, tasks = validate(data)
    fields = ["id", "milestone", "release", "title", "area", "status", "depends_on", "deliverable", "acceptance", "evidence"]
    with (ROOT / "rust-backlog.csv").open("w", encoding="utf-8-sig", newline="") as stream:
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
    (ROOT / "ROADMAP.html").write_text(HTML.replace("__DATA__", payload), encoding="utf-8")
    counts = Counter(task["milestone"] for task in tasks.values())
    print(f"Validated {len(tasks)} tasks across {len(milestones)} milestones; dependency graph is acyclic.")
    print("Tasks by stage:", ", ".join(f"{key}={counts[key]}" for key in milestones))
    print("Generated ROADMAP.html and rust-backlog.csv")


if __name__ == "__main__":
    main()
