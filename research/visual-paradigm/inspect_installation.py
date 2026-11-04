"""Read-only installation metadata inspection; no application code is extracted."""
from pathlib import Path
from collections import Counter
import zipfile, json, hashlib, struct, re
import sqlite3, xml.etree.ElementTree as ET, argparse

parser = argparse.ArgumentParser()
parser.add_argument('--root',default=r'C:\Program Files\Visual Paradigm 18.1')
parser.add_argument('--out',default=str(Path(__file__).parent/'evidence'))
args=parser.parse_args()
ROOT = Path(args.root)
OUT = Path(args.out)
OUT.mkdir(exist_ok=True,parents=True)
files = sorted(p for p in ROOT.rglob('*') if p.is_file())
inventory = [{'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size} for p in files]
(OUT/'files.json').write_text(json.dumps(inventory,indent=2),encoding='utf-8')
groups = {}
for f in inventory:
    key = f['path'].split('/')[0]
    g = groups.setdefault(key, {'files':0,'bytes':0})
    g['files']+=1; g['bytes']+=f['bytes']
archives=[]
api=[]
for p in files:
    if p.suffix.lower() != '.jar': continue
    try:
        with zipfile.ZipFile(p) as z:
            names=z.namelist(); classes=[n for n in names if n.endswith('.class')]
            prefixes=Counter('/'.join(n.split('/')[:3]) for n in classes)
            manifest=z.read('META-INF/MANIFEST.MF').decode('utf-8',errors='replace') if 'META-INF/MANIFEST.MF' in names else ''
            item={'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'entries':len(names),'classes':len(classes),'top_packages':prefixes.most_common(18),'manifest':manifest[:4000]}
            archives.append(item)
            if p.relative_to(ROOT).as_posix() == 'lib/openapi.jar': api=names
    except zipfile.BadZipFile: archives.append({'path':str(p),'error':'not a ZIP archive'})
(OUT/'archives.json').write_text(json.dumps(archives,indent=2),encoding='utf-8')
(OUT/'public-api-entries.txt').write_text('\n'.join(api),encoding='utf-8')
pe=[]
for p in (ROOT/'bin').glob('*.exe'):
    data=p.read_bytes(); off=struct.unpack_from('<I',data,0x3c)[0]
    machine=struct.unpack_from('<H',data,off+4)[0] if data[off:off+4]==b'PE\0\0' else None
    pe.append({'path':p.relative_to(ROOT).as_posix(),'machine':hex(machine) if machine else None,'sha256':hashlib.sha256(data).hexdigest()})
samples=[]
for p in files:
    if p.suffix.lower()!='.vpp': continue
    with p.open('rb') as stream: head=stream.read(64)
    s={'path':p.relative_to(ROOT).as_posix(),'bytes':p.stat().st_size,'header_hex':head.hex(),'header_ascii':''.join(chr(c) if 32<=c<127 else '.' for c in head)}
    if zipfile.is_zipfile(p):
        with zipfile.ZipFile(p) as z: s['zip_entries']=z.namelist()[:30]
    samples.append(s)
summary={'root':str(ROOT),'file_count':len(files),'bytes':sum(f['bytes'] for f in inventory),'directories':groups,'extensions':Counter(p.suffix.lower() for p in files).most_common(25),'launchers':pe,'vpp_samples':samples}
(OUT/'summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
db=sqlite3.connect((ROOT/'resources/BaggageSchemas.vpp').as_uri()+'?mode=ro',uri=True)
schema=[]
for name,sql in db.execute("select name,sql from sqlite_master where type='table' order by name"):
    quoted='"'+name.replace('"','""')+'"'
    columns=[{'name':r[1],'type':r[2]} for r in db.execute('pragma table_info('+quoted+')')]
    count=db.execute('select count(*) from '+quoted).fetchone()[0]
    schema.append({'table':name,'sql':sql,'columns':columns,'rows':count})
db.close()
(OUT/'bundled-project-schema.json').write_text(json.dumps(schema,indent=2),encoding='utf-8')
xs=ET.parse(ROOT/'bundled/project.xsd').getroot()
elements=[e.attrib['name'] for e in xs.findall('{http://www.w3.org/2001/XMLSchema}element')]
(OUT/'xml-export-elements.txt').write_text('\n'.join(elements),encoding='utf-8')
print('Bundled SQLite project tables:',json.dumps([{'name':s['table'],'rows':s['rows']} for s in schema]))
print(json.dumps({'files':len(files),'bytes':summary['bytes'],'directories':groups,'launchers':pe,'vpp_samples':samples[:12],'api_entries':len(api),'archives':[{k:a[k] for k in ('path','classes') if k in a} for a in archives if a.get('classes',0)>1000]},indent=2))
