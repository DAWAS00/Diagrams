"""Build a local, dependency-free research evidence viewer."""
from pathlib import Path
from collections import defaultdict
import json, html

root=Path(__file__).parent
families=defaultdict(list)
for item in json.loads((root/'ce-evidence/diagram-families.json').read_text()):
    families[item['family']].append(item['constant'].removeprefix('DIAGRAM_TYPE_').replace('_',' ').title())
captions={
 'class-editor':'Class palette, inline name and selection handles',
 'class-association':'Two semantic classes joined by an association',
 'sequence-editor':'Lifelines, activation bars and numbered message',
 'er-editor':'ER palette and conceptual/logical/physical modes',
 'er-specification':'Entity specification: schema, domain and model level',
 'er-columns':'Structured column table and primary-key settings',
 'model-explorer':'Semantic model hierarchy',
 'project-browser':'Diagram thumbnails grouped by notation',
 'view-ribbon':'Workspace, viewport, grid, layers and styles',
 'modeling-ribbon':'Modeling feature entrypoints; not all executed',
 'tools-ribbon':'Tooling feature entrypoints; not all executed',
 'agile-ribbon':'Agile feature entrypoints; not executed',
 'itsm-ribbon':'ITSM feature entrypoints; not executed',
 'team-ribbon':'Repository connection entrypoint; not invoked',
 'export-menu':'Exchange and image export entrypoints',
 'xml-export':'Simple XML export selection: all three research diagrams',
}
cards=''.join(f'<figure><a href="ce-evidence/ui/{p.name}" target="_blank"><img loading="lazy" src="ce-evidence/ui/{p.name}" alt="{html.escape(captions.get(p.stem,p.stem))}"></a><figcaption>{html.escape(captions.get(p.stem,p.stem))}</figcaption></figure>' for p in sorted((root/'ce-evidence/ui').glob('*.png')))
groups=''.join(f'<details class="family"><summary>{html.escape(name)} <span>{len(types)}</span></summary><ul>'+''.join(f'<li>{html.escape(t)}</li>' for t in types)+'</ul></details>' for name,types in sorted(families.items()))
page='''<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Visual Paradigm CE — diagram and UI evidence</title>
<style>:root{font:16px/1.6 system-ui;color:#e5edf8;background:#101721}*{box-sizing:border-box}body{margin:0}main{max-width:1300px;margin:auto;padding:44px 24px}h1{font-size:clamp(28px,4vw,46px);line-height:1.15;max-width:950px}h2{margin-top:42px}p{max-width:950px;color:#bdcadb}a{color:#91c8ff}nav{display:flex;flex-wrap:wrap;gap:16px}.stats{display:flex;flex-wrap:wrap;gap:12px;margin:28px 0}.stats div{border:1px solid #344253;padding:14px 22px;border-radius:12px}.stats b{display:block;font-size:28px;color:#8de0d0}.notice{border-left:3px solid #edbd6e;padding:8px 18px;background:#1d2530}.gallery{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,450px),1fr));gap:20px}figure{margin:0;border:1px solid #344253;border-radius:12px;overflow:hidden;background:#1a2430}img{display:block;width:100%;height:auto}figcaption{padding:12px 16px}details{border:1px solid #344253;border-radius:8px;margin:8px 0;padding:10px 16px}summary{cursor:pointer}summary span{float:right;color:#8de0d0}input{font:inherit;background:#1a2430;color:#fff;border:1px solid #52657d;border-radius:8px;padding:12px;width:100%;max-width:650px}li{color:#c7d4e4}code{overflow-wrap:anywhere}.steps{display:flex;flex-wrap:wrap;gap:10px}.steps span{padding:12px;background:#233548;border-radius:8px}</style>
<main><nav><a href="UI-STUDY.md">UI study</a><a href="DIAGRAM-CATALOG.md">Full catalog</a><a href="PLANNING-BACKLOG.md">Planning backlog</a><a href="DOCUMENTED-WORKFLOWS.md">Documented workflows</a></nav>
<p>LOCAL RESEARCH · 4 NOVEMBER 2026</p><h1>Understanding the diagram engine and its workspace</h1><p>Visual Paradigm Community Edition 18.1. Observed UI, saved project evidence, and a complete inventory of installed public diagram-type constants.</p>
<div class="stats"><div><b>101</b>shared API constants</div><div><b>17</b>diagram families</div><div><b>3</b>native editors exercised</div><div><b>16</b>UI captures</div></div>
<p class="notice">The catalog is broader than enabled CE functionality. Screenshots prove observed UI surfaces; they do not prove every displayed command works. Native implementation algorithms were not decompiled. See the coverage ledger before making parity claims.</p>
<h2>The architecture to reproduce independently</h2><div class="steps"><span>Project & semantic model</span><span>Separate diagram views</span><span>Shared canvas + commands</span><span>Notation-specific editors</span><span>Save & exchange</span></div>
<p>Class members, sequence timing and ER column/key data require different semantics and property editors. Geometry, navigation, selection, styles and persistence can share a common foundation.</p>
<h2>Observed interface</h2><p>Click any capture for its full-size local image. No external resources are loaded by this page.</p><div class="gallery">'''+cards+'''</div>
<h2>All installed diagram families</h2><p>Filter to explore the inventory. Every entry is an API observation, not an assertion of CE runtime availability.</p><label for="filter">Filter families or diagram types</label><br><input id="filter" type="search" placeholder="For example: sequence, requirement, database"><section id="families">'''+groups+'''</section>
<h2>Reproducible evidence</h2><ul><li><a href="ce-evidence/ui-research.vpp">Native research project</a></li><li><a href="ce-evidence/ui-project-inspection.json">SQLite model/view inspection</a></li><li><a href="ce-evidence/xml-export/project.xml">Exported project XML</a></li><li><a href="ce-evidence/xml-export-inspection.json">XML reference and geometry inspection</a></li></ul></main>
<script>document.getElementById('filter').addEventListener('input',e=>{const q=e.target.value.toLowerCase();document.querySelectorAll('.family').forEach(d=>{d.hidden=!d.textContent.toLowerCase().includes(q);d.open=!!q&&!d.hidden;});});</script></html>'''
(root/'UI-GALLERY.html').write_text(page,encoding='utf-8')
assert len(captions)==len(list((root/'ce-evidence/ui').glob('*.png')))
assert sum(map(len,families.values()))==101
print('Wrote gallery: 16 screenshots; 101 constants across',len(families),'families.')
