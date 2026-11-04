"""Read-only inspection of the disposable CE project XML export."""
from pathlib import Path
from collections import Counter
import hashlib, json, zipfile
import xml.etree.ElementTree as ET

BASE=Path(__file__).parent/'ce-evidence'
SOURCE=BASE/'xml-export'/'project.xml'
root=ET.parse(SOURCE).getroot()
models=root.find('Models')
diagrams=root.find('Diagrams')
model_ids={e.attrib['Id']:e for e in models.iter() if 'Id' in e.attrib}
geometry_keys={'X','Y','Width','Height','ZOrder','BaseY','Points','From','To','Model','MetaModelElement','Id','Name','MasterView','Parent','ParentId','FromShapeXDiff','FromShapeYDiff','ToShapeXDiff','ToShapeYDiff','FromPinType','ToPinType','ConnectorStyle','ConnectionPointType','ConnectToPoint'}
identity_keys={'Id','Name','Type','From','To','EndModelElement','EndRelationshipFromMetaModelElement','EndRelationshipToMetaModelElement','FromActivation','ToActivation','Direction','Navigable','Multiplicity','AggregationKind','Model','MetaModelElement','MasterView','MasterViewId','BaseY','X','Y','Width','Height','SequenceNumber','DataModel','Visibility'}
def attrs(e, keys): return {k:v for k,v in e.attrib.items() if k in keys}
def refs(e):
    return [{'tag':c.tag,'attrs':dict(c.attrib)} for c in e.iter() if c is not e and any(k in c.attrib for k in ['Idref','IdRef','Ref','EndModelElement'])]

diagram_records=[]
for d in diagrams:
    views=[]
    for group in d:
        if group.tag not in ('Shapes','Connectors'):continue
        for v in group.iter():
            if v is group or 'Model' not in v.attrib: continue
            view={'tag':v.tag,'kind':group.tag,'attrs':attrs(v,geometry_keys),'model_reference_resolves':v.attrib['Model'] in model_ids}
            view['geometry_children']=[{'tag':c.tag,'attrs':dict(c.attrib),'text':c.text.strip() if c.text and c.text.strip() else None} for c in v.iter() if c is not v and any(s in c.tag.lower() for s in ['point','anchor','rect','bound'])]
            views.append(view)
    diagram_records.append({'type':d.tag,'attrs':attrs(d,geometry_keys|{'ZoomRatio','GridVisible','AlignToGrid','GridWidth','GridHeight','Editable','DataModel','AutoFitShapesSize'}),'views':views})
relevant=[]
for e in models.iter():
    if e.tag not in {'Class','Association','AssociationEnd','Message','MessageEnd','DBTable','InteractionLifeLine','Activation','Frame'} or 'Id' not in e.attrib:continue
    relevant.append({'tag':e.tag,'attrs':attrs(e,identity_keys),'reference_children':refs(e)})
options=root.find('./ProjectInfo/ProjectOptions/DiagramOptions')
render_option_names=['FractionalMetrics','GraphicAntiAliasing','TextAntiAliasing','EnableMinimumSize','DefaultDiagramBackground','DefaultConnectorStyle','DefaultConnectionPointStyle','DefaultLineJumps','LineJumpSize','ShowNumberInSequenceDiagram','ShowNumberInCollaborationDiagram','PaintConnectorThroughLabel','PointConnectorEndToCompartmentMember','SnapConnectorsAfterZoom','DefaultColumnType','AutoSetColumnType','AutoSetAttributeType']
data_zip=BASE/'xml-export'/'data.zip'
with zipfile.ZipFile(data_zip) as z:
    archive=[{'path':i.filename,'bytes':i.file_size,'compressed_bytes':i.compress_size} for i in z.infolist()]
summary={'source':str(SOURCE),'source_bytes':SOURCE.stat().st_size,'source_sha256':hashlib.sha256(SOURCE.read_bytes()).hexdigest(),'root_tag':root.tag,'root_attributes':dict(root.attrib),'root_sections':[e.tag for e in root],'counts':{'top_level_model_types':dict(Counter(e.tag for e in models)),'id_bearing_model_types_recursive':dict(Counter(e.tag for e in models.iter() if 'Id' in e.attrib)),'diagram_types':dict(Counter(e.tag for e in diagrams)),'visual_types':dict(Counter(v['tag'] for d in diagram_records for v in d['views']))},'rendering_and_defaults':{k:options.attrib[k] for k in render_option_names if k in options.attrib},'diagrams':diagram_records,'relevant_semantic_models':relevant,'data_zip':{'source':str(data_zip),'bytes':data_zip.stat().st_size,'sha256':hashlib.sha256(data_zip.read_bytes()).hexdigest(),'entries':archive},'limitations':['ExporterVersion 16.1.1 is the XML exporter version, not the installed application version (18.1).','This fixture covers class, sequence and ER diagrams only, not every supported notation.','Option values are observed serialization; numeric and escaped-character enumerations are not decoded unless separately established.','All semantic model references on inventoried views resolve in this export; this does not prove import round-trip compatibility.']}
out=BASE/'xml-export-inspection.json'
out.write_text(json.dumps(summary,indent=2),encoding='utf-8')
print(json.dumps({'output':str(out),'diagram_types':summary['counts']['diagram_types'],'visual_types':summary['counts']['visual_types'],'all_view_models_resolve':all(v['model_reference_resolves'] for d in diagram_records for v in d['views']),'zip_entries':len(archive)},indent=2))
