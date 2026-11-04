"""Group public API names for research planning; no editor enablement is inferred."""
from pathlib import Path
import json

ROOT=Path(__file__).parent
names=(ROOT/'ce-evidence/diagram-types.txt').read_text().splitlines()
groups={
'UML structural': 'CLASS_DIAGRAM OBJECT_DIAGRAM PACKAGE_DIAGRAM COMPONENT_DIAGRAM DEPLOYMENT_DIAGRAM COMPOSITE_STRUCTURE_DIAGRAM PROFILE_DIAGRAM',
'UML behavioral': 'ACTIVITY_DIAGRAM COMMUNICATION_DIAGRAM INTERACTION_DIAGRAM INTERACTION_OVERVIEW_DIAGRAM STATE_DIAGRAM TIMING_DIAGRAM USE_CASE_DIAGRAM',
'SysML and requirements': 'BLOCK_DEFINITION_DIAGRAM INTERNAL_BLOCK_DIAGRAM PARAMETRIC_DIAGRAM REQUIREMENT_DIAGRAM',
'Services and code-oriented models': 'SOA_ML_SERVICES_ARCHITECTURE_DIAGRAM SOA_ML_SERVICE_CATEGORIZATION_DIAGRAM SOA_ML_SERVICE_CONTRACT_DIAGRAM SOA_ML_SERVICE_INTERFACE_DIAGRAM SOA_ML_SERVICE_PARTICIPANT_DIAGRAM WSDL_DIAGRAM EJB_DIAGRAM',
'Database and ORM': 'CHEN_NOTATION_ERDUI_MODEL ER_DIAGRAM ORM_DIAGRAM',
'Business processes and cases': 'BUSINESS_PROCESS_DIAGRAM BUSINESS_PROCESS_REENGINEERING CONVERSATION_DIAGRAM CMMN_DIAGRAM EPC_DIAGRAM PROCESS_MAP_DIAGRAM PM_PROCESS_MAP',
'Data-flow models': 'DATA_FLOW_DIAGRAM GANE_SARSON_DFDUI_MODEL YOURDON_AND_COAD_DFDUI_MODEL YOURDON_DE_MARCO_DFDUI_MODEL',
'Enterprise architecture': 'ARCHI_MATE4_DIAGRAM ARCHI_MATE_DIAGRAM ARCHITECTURE_ROADMAP MIGRATION_ROADMAP ZACHMAN_DIAGRAM EA_FORM_DIAGRAM EA_PROCESS_STEP_DIAGRAM OVERVIEW_DIAGRAM',
'Rules, facts and decisions': 'BUSINESS_MOTIVATION_MODEL_DIAGRAM BUSINESS_RULE_EDITOR_DIAGRAM DECISION_MODEL_DIAGRAM DTB_DECISION_TABLE_EDITOR_DIAGRAM FACT_DIAGRAM FACT_TYPE_EDITOR_DIAGRAM TERM_EDITOR_DIAGRAM',
'Analysis and specification': 'ANALYSIS_DIAGRAM ANALYSIS_ITEM_DIAGRAM COMPETITOR_ANALYSIS CRC_CARD_DIAGRAM MATURITY_ANALYSIS ROOT_CAUSE_ANALYSIS TEXTUAL_ANALYSIS USE_CASE_DESCRIPTION_DIAGRAM',
'Planning and experience': 'PERT_CHART BREAKDOWN_STRUCTURE USER_STORY_MAP_DIAGRAM CUSTOMER_JOURNEY_MAP',
'General visual diagrams': 'BRAINSTORM MIND_MAP_DIAGRAM FISHBONE_DIAGRAM ORGANIZATION_CHART FLOWCHART FREEHAND_DIAGRAM FREEHAND_DIAGRAM2',
'Structured canvases': 'PROCESS_CANVAS QUALITY_CONTROL_CANVAS',
'Data and tabular editors': 'CHART_DIAGRAM DATA_STUDIO_SHEET DATA_TABLE_DIAGRAM ETL_TABLE_DIAGRAM GRID_DIAGRAM MATRIX_DIAGRAM TABULAR',
'UI and wireflow': 'USER_INTERFACE_DIAGRAM WEB_DIAGRAM WFI_PAD_DIAGRAM WFI_PHONE_DIAGRAM WF_ANDROID_PHONE_DIAGRAM WF_ANDROID_TABLET_DIAGRAM WF_DESKTOP_DIAGRAM WF_WEB_DIAGRAM WIREFLOW_DIAGRAM',
'Documents and form editors': 'DOCUMENTATION_CABINET REPORT_DIAGRAM FORM VIEW_EDITOR_DIAGRAM',
'C4 architecture': 'C4_MODEL_COMPONENT_DIAGRAM C4_MODEL_CONTAINER_DIAGRAM C4_MODEL_DEPLOYMENT_DIAGRAM C4_MODEL_DYNAMIC_DIAGRAM C4_MODEL_SYSTEM_CONTEXT_DIAGRAM C4_MODEL_SYSTEM_LANDSCAPE_DIAGRAM',
}
mapping={}
for group, values in groups.items():
    for value in values.split():
        name='DIAGRAM_TYPE_'+value
        assert name not in mapping,name
        mapping[name]=group
assert set(mapping)==set(names),(set(names)-set(mapping),set(mapping)-set(names))
entries=[{'constant':name,'family':mapping[name],'evidence':'installed public API constant','ce_runtime_status':'not established by API presence'} for name in names]
(ROOT/'ce-evidence/diagram-families.json').write_text(json.dumps(entries,indent=2),encoding='utf-8')
doc=ROOT/'DIAGRAM-CATALOG.md'
with doc.open('a',encoding='utf-8') as f:
    f.write('\n## Exhaustive public-constant coverage appendix\n\nAll 101 installed constants appear exactly once below. Family assignment is an analytical grouping, not the original application\'s menu taxonomy. Constant names are retained verbatim, with the common `DIAGRAM_TYPE_` prefix omitted for readability.\n\n')
    for group,values in groups.items():
        f.write('### '+group+'\n\n'+', '.join('`'+x+'`' for x in values.split())+'.\n\n')
print(f'Validated {len(entries)} constants in {len(groups)} families; wrote diagram-families.json and catalog appendix.')
