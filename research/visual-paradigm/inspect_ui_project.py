"""Read-only metadata inspection of a disposable VP CE UI research project.

Uses a SQLite read transaction and records pre/post file identity. Rerun after
saving UI changes; a changing file is explicitly marked rather than treated as
a stable fixture. Definition prefixes are observations, not a decoded grammar.
"""
from pathlib import Path
from collections import Counter
import argparse, datetime, hashlib, json, sqlite3, re

BASE=Path(__file__).parent
parser=argparse.ArgumentParser()
parser.add_argument('--project',default=str(BASE/'ce-evidence/ui-research.vpp'))
parser.add_argument('--out',default=str(BASE/'ce-evidence/ui-project-inspection.json'))
args=parser.parse_args()
project=Path(args.project).resolve()
out=Path(args.out).resolve()

def identity(path):
    if not path.exists(): return None
    data=path.read_bytes()
    stat=path.stat()
    return {'path':str(path),'bytes':len(data),'mtime_ns':stat.st_mtime_ns,'sha256':hashlib.sha256(data).hexdigest()}

before=identity(project)
sidecars_before={suffix:identity(Path(str(project)+suffix)) for suffix in ('-wal','-shm','-journal')}
db=sqlite3.connect(project.as_uri()+'?mode=ro',uri=True)
db.row_factory=sqlite3.Row
db.execute('PRAGMA query_only=ON')
db.execute('BEGIN')
tables=db.execute("SELECT name,sql FROM sqlite_master WHERE type='table' ORDER BY name").fetchall()
result={'inspected_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'method':'SQLite mode=ro, query_only=ON, read transaction; no mutations','project':before,'sidecars_before':sidecars_before,'quick_check':[r[0] for r in db.execute('PRAGMA quick_check')],'schema':[],'models':[],'diagrams':[],'views':[]}

def quoted(s): return '"'+s.replace('"','""')+'"'
def as_bytes(value):
    if value is None: return b''
    return value.encode('utf-8') if isinstance(value,str) else bytes(value)
def definition_info(value,limit=192):
    data=as_bytes(value)
    return {'bytes':len(data),'prefix_hex':data[:32].hex(),'prefix_text':data[:limit].decode('utf-8',errors='replace'),'prefix_limit_bytes':limit,'truncated':len(data)>limit}
def safe_row(row):
    return {k:(definition_info(v) if isinstance(v,bytes) or k=='DEFINITION' else v) for k,v in dict(row).items()}

names={r['name'] for r in tables}
for table in tables:
    name=table['name']
    result['schema'].append({'table':name,'sql':table['sql'],'rows':db.execute('SELECT count(*) FROM '+quoted(name)).fetchone()[0],'columns':[dict(r) for r in db.execute('PRAGMA table_info('+quoted(name)+')')]})

model_rows=db.execute('SELECT * FROM MODEL_ELEMENT ORDER BY MODEL_TYPE,NAME,ID').fetchall() if 'MODEL_ELEMENT' in names else []
ids={str(r['ID']):{'id':r['ID'],'type':r['MODEL_TYPE'],'name':r['NAME']} for r in model_rows}
result['model_type_counts']=dict(Counter(r['MODEL_TYPE'] for r in model_rows))
for row in model_rows:
    entry=safe_row(row)
    result['models'].append(entry)

for key,table,order in [('diagrams','DIAGRAM','DIAGRAM_TYPE,NAME,ID'),('views','DIAGRAM_ELEMENT','DIAGRAM_ID,ID')]:
    if table in names:
        result[key]=[safe_row(r) for r in db.execute('SELECT * FROM '+quoted(table)+' ORDER BY '+order)]

result['view_serialized_fields']=[]
if 'DIAGRAM_ELEMENT' in names:
    for row in db.execute('SELECT * FROM DIAGRAM_ELEMENT ORDER BY DIAGRAM_ID,ID'):
        text=as_bytes(row['DEFINITION']).decode('utf-8',errors='replace')
        fields={m.group(1):m.group(2) for m in re.finditer(r'^\t(x|y|width|height|_points|_fromShape|_toShape|metaModelElement|fromPinType|toPinType)=([^\n]*);$',text,re.MULTILINE)}
        result['view_serialized_fields'].append({'id':row['ID'],'shape_type':row['SHAPE_TYPE'],'model_id':row['MODEL_ELEMENT_ID'],'observed_top_level_fields':fields,'limit':'Literal single-line assignments only; geometry coordinate semantics not inferred.'})

result['relationship_candidates']=[]
relationship_words=('association','generalization','dependency','realization','flow','relationship','include','extend','message','transition','foreignkey','link')
for row in model_rows:
    model_type=str(row['MODEL_TYPE'])
    if any(word in model_type.lower() for word in relationship_words):
        data=as_bytes(row['DEFINITION'])
        references=[item for mid,item in ids.items() if mid!=str(row['ID']) and mid.encode('utf-8') in data]
        result['relationship_candidates'].append({'id':row['ID'],'type':model_type,'name':row['NAME'],'definition':definition_info(row['DEFINITION'],1024),'exact_id_occurrences_in_definition':references,'interpretation_limit':'Literal identifier occurrences are candidate references; source/target roles and grammar are not inferred from occurrence order.'})
        text=data.decode('utf-8',errors='replace')
        ends={}
        for match in re.finditer(r'^\t(from|to)=\{(.*?)^\t\}\};',text,re.MULTILINE|re.DOTALL):
            ends[match.group(1)]={field:mid for field,mid in re.findall(r'^\t\t(type|EndModelElement)=<([^>]+)>;',match.group(2),re.MULTILINE)}
        if ends: result['relationship_candidates'][-1]['observed_endpoint_field_references']=ends

result['diagram_type_counts']=dict(Counter(str(r.get('DIAGRAM_TYPE')) for r in result['diagrams']))
result['view_shape_type_counts']=dict(Counter(str(r.get('SHAPE_TYPE')) for r in result['views']))
result['repeated_model_view_references']=[]
counts=Counter(str(r.get('MODEL_ELEMENT_ID')) for r in result['views'] if r.get('MODEL_ELEMENT_ID') is not None)
for model_id,count in counts.items():
    if count>1: result['repeated_model_view_references'].append({'model':ids.get(model_id,{'id':model_id}),'view_count':count})
db.rollback()
db.close()
after=identity(project)
result['project_after']=after
result['sidecars_after']={suffix:identity(Path(str(project)+suffix)) for suffix in ('-wal','-shm','-journal')}
result['file_unchanged_during_inspection']=before==after
result['stable_file_and_wal_identity']=before==after and sidecars_before['-wal']==result['sidecars_after']['-wal']
result['limits']=['This reads a SQLite transaction from a potentially running application; a stable fixture should be saved and quiescent.','BLOB prefixes are bounded and the complete definition grammar is not decoded.','Relationship candidates are classified by type-name terms and require semantic corroboration.','SHA-256 identifies the main file; any WAL identity is recorded separately.']
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(result,indent=2,ensure_ascii=False),encoding='utf-8')
print(json.dumps({'output':str(out),'sha256':before['sha256'],'unchanged':result['file_unchanged_during_inspection'],'stable_file_and_wal_identity':result['stable_file_and_wal_identity'],'quick_check':result['quick_check'],'diagrams':[(r.get('NAME'),r.get('DIAGRAM_TYPE')) for r in result['diagrams']],'named_models':[(r.get('NAME'),r.get('MODEL_TYPE')) for r in result['models'] if r.get('NAME') in ('Customer','Order')],'relationship_candidates':[{k:r[k] for k in ('id','type','name','exact_id_occurrences_in_definition')} for r in result['relationship_candidates']],'views':len(result['views'])},indent=2))
