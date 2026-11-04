"""Reproducible comparison of installed VP constants with the unbuilt Rust plan."""
from pathlib import Path
from collections import Counter
import csv,json

ROOT=Path(__file__).resolve().parent
PROJECT=ROOT.parent.parent
source=PROJECT/'research/visual-paradigm/ce-evidence/diagram-families.json'
entries=json.loads(source.read_text(encoding='utf-8'))
constants=(PROJECT/'research/visual-paradigm/ce-evidence/diagram-types.txt').read_text().splitlines()
assert len(entries)==101
assert len({e['constant'] for e in entries})==101
assert {e['constant'] for e in entries}==set(constants)

family_semantics={
'UML structural':('Typed structural elements, semantic containment, compartments/ports and relationship-end properties.','High'),
'UML behavioral':('Behavior-specific control/state/message semantics and constrained geometry/order.','High to very high'),
'SysML and requirements':('Typed blocks/ports/constraints or requirement text and traceability; notation-specific relation validation.','High'),
'Services and code-oriented models':('Service/code schemas, typed interfaces/contracts/participants and linked definitions.','Very high'),
'Database and ORM':('Typed tables/columns/keys/cardinality or ORM mappings; row-level properties and references.','High to very high'),
'Business processes and cases':('Process/case objects, lane/container membership and flow admissibility; notation-specific rules.','Very high'),
'Data-flow models':('Typed processes/stores/actors/flows, decomposition and notation-specific glyphs/rules.','High'),
'Enterprise architecture':('Architecture element/relation taxonomy, viewpoint filters and constrained framework views.','Very high'),
'Rules, facts and decisions':('Terms/facts/rules/decisions and typed tabular properties; execution semantics require separate specification.','Very high'),
'Analysis and specification':('Structured analysis/specification records, linked references and specialized text/table views.','High'),
'Planning and experience':('Planning/story/journey objects, constrained lanes/order or network/tree structure.','High to very high'),
'General visual diagrams':('Family-specific glyphs, branch/tree or freeform structure and connection/layout rules.','Medium to high'),
'Structured canvases':('Fixed sections and movable cards/objects; template and section-aware persistence/export.','Medium to high'),
'Data and tabular editors':('Cell/grid/matrix/chart data model, typed editing and view-specific calculations/mappings.','Very high'),
'UI and wireflow':('Widget/screen definitions, nested frames, screen flow and device-specific layout.','Very high'),
'Documents and form editors':('Document/form templates, property binding, rich text or page-aware composition.','Very high'),
'C4 architecture':('Context/container/component/deployment hierarchy, boundaries and dynamic-step views.','High'),
}
specific={
'CLASS_DIAGRAM':'Class identity; ordered attributes/operations; associations with roles/multiplicity; generalization; independent views. Missing wider UML classifiers and relation kinds remain out of scope.',
'OBJECT_DIAGRAM':'Instances, classifier references, slots/values and instance links; class-shaped rendering does not supply these semantics.',
'PACKAGE_DIAGRAM':'Package identity/ownership, nested or imported packages, dependency/import relations and independent visual containment.',
'COMPONENT_DIAGRAM':'Components, provided/required interfaces, ports and assembly/delegation connections.',
'DEPLOYMENT_DIAGRAM':'Nodes/devices/execution environments, deployed artifacts and communication paths.',
'COMPOSITE_STRUCTURE_DIAGRAM':'Structured classifiers, parts/roles, ports, connectors and internal containment.',
'PROFILE_DIAGRAM':'Stereotype/profile definitions, metaclass extension and tagged-value schema rules.',
'USE_CASE_DIAGRAM':'Actors/use cases/system boundaries, associations, include/extend and generalization rules.',
'ACTIVITY_DIAGRAM':'Actions/nodes/control and object flows, partitions, containment and edge admissibility.',
'STATE_DIAGRAM':'States/regions, pseudostates, transitions/guards/actions and hierarchical containment.',
'INTERACTION_DIAGRAM':'Sequence lifelines, ordered messages, activation spans, frames/fragments and specialized movement/numbering.',
'COMMUNICATION_DIAGRAM':'Object roles/links and ordered or nested message numbering independent of freeform connector layout.',
'TIMING_DIAGRAM':'Time axes, state/value tracks, transitions and timing/duration constraints.',
'INTERACTION_OVERVIEW_DIAGRAM':'Control-flow overview referencing interactions with branching and sequencing semantics.',
'ER_DIAGRAM':'Entity/table columns, type/length, PK/FK/nullability/uniqueness, cardinality, indices/constraints and model level.',
'CHEN_NOTATION_ERDUI_MODEL':'Chen-specific entity/relationship/attribute glyphs, participation/cardinality and model mapping; not implied by M5 table notation.',
'ORM_DIAGRAM':'Object/relational mapping records and transformation links; not the v0.1 class diagram schema.',
'REQUIREMENT_DIAGRAM':'Requirement IDs/text, hierarchy and typed trace/satisfy/verify/derive relationships.',
'BLOCK_DEFINITION_DIAGRAM':'Blocks, value/part/reference properties, associations and SysML-specific ports/types.',
'INTERNAL_BLOCK_DIAGRAM':'Block-context parts/ports/connectors and internal containment with referenced definitions.',
'PARAMETRIC_DIAGRAM':'Constraint blocks/properties, parameters, binding connectors and typed value references.',
'FLOWCHART':'Process/decision/terminal/input-output shapes, labeled control flow and constrained connection tools.',
'BUSINESS_PROCESS_DIAGRAM':'BPMN task/event/gateway/pool/lane objects, flow kinds, boundary events and subprocess containment.',
'CONVERSATION_DIAGRAM':'BPMN conversation/participant/link structures; not a generic UML message view.',
'CMMN_DIAGRAM':'Case plan items, stages, sentries and case-specific event/entry/exit conditions.',
'DECISION_MODEL_DIAGRAM':'Decisions/input data/knowledge references and dependency structures; evaluation engine is separate scope.',
'DTB_DECISION_TABLE_EDITOR_DIAGRAM':'Typed decision-table rows/columns, conditions/actions and rule-order semantics; execution behavior unresolved.',
'MIND_MAP_DIAGRAM':'Root/branch hierarchy, collapse/expand and branch-aware layout.',
'ORGANIZATION_CHART':'Organizational hierarchy and branch-aware placement/reparenting.',
'PERT_CHART':'Activity/dependency network and any time/schedule calculations requiring explicit specification.',
'USER_STORY_MAP_DIAGRAM':'Activities/tasks/stories, ordered lanes/releases and card movement.',
'CUSTOMER_JOURNEY_MAP':'Stages/touchpoints, lanes, customer experience annotations and structured cells.',
'WIREFLOW_DIAGRAM':'Screen/widget views plus navigation connections and screen-aware layout.',
'CHART_DIAGRAM':'Typed chart data, series/axes/legend, data editing and chart-specific export.',
}
milestone4={'FLOWCHART','USE_CASE_DIAGRAM'}
milestone5={'ACTIVITY_DIAGRAM','STATE_DIAGRAM','INTERACTION_DIAGRAM','ER_DIAGRAM','REQUIREMENT_DIAGRAM','BLOCK_DEFINITION_DIAGRAM','INTERNAL_BLOCK_DIAGRAM','PARAMETRIC_DIAGRAM'}
uml_documented={'CLASS_DIAGRAM','OBJECT_DIAGRAM','PACKAGE_DIAGRAM','COMPONENT_DIAGRAM','DEPLOYMENT_DIAGRAM','COMPOSITE_STRUCTURE_DIAGRAM','ACTIVITY_DIAGRAM','STATE_DIAGRAM','INTERACTION_DIAGRAM','COMMUNICATION_DIAGRAM','TIMING_DIAGRAM','INTERACTION_OVERVIEW_DIAGRAM','USE_CASE_DIAGRAM'}
ce_documented=uml_documented|milestone5|{'CHEN_NOTATION_ERDUI_MODEL','FLOWCHART'}
not_marked_ce={'BUSINESS_PROCESS_DIAGRAM','CONVERSATION_DIAGRAM','DATA_FLOW_DIAGRAM','GANE_SARSON_DFDUI_MODEL','YOURDON_AND_COAD_DFDUI_MODEL','YOURDON_DE_MARCO_DFDUI_MODEL'}
rows=[]
for e in entries:
    short=e['constant'].removeprefix('DIAGRAM_TYPE_')
    family=e['family']
    semantics,cost=family_semantics[family]
    semantics=specific.get(short,semantics+' Specific subtype behavior remains to be established.')
    if short=='CLASS_DIAGRAM':
        coverage='Planned partial class subset; not implemented'
        milestone='M2-M3 / v0.1'
        gap='Core class/member/association/generalization workflow only; wider UML class notation, profile support and advanced presentation not included.'
    elif short in milestone4:
        coverage='Absent from v0.1; not implemented'
        milestone='M4 planned notation adapter'
        gap='All notation-specific elements/relations/inspectors/validation/export fixtures absent from v0.1.'
    elif short in milestone5:
        coverage='Absent from v0.1; not implemented'
        milestone='M5 planned family milestone; detailed subset TBD'
        gap='Specialized schema, property editors, interaction/layout rules and fixtures deferred; M5 is not a full-standard parity commitment.'
    else:
        coverage='Absent from v0.1; not implemented'
        milestone='M6 candidate only; no committed scope'
        gap='Entire editor deferred without a committed implementation proposal, full semantics or acceptance fixtures.'
    if short in {'CLASS_DIAGRAM','INTERACTION_DIAGRAM','ER_DIAGRAM'}:
        ce='Basic local CE UI workflow observed; advanced behavior and complete notation remain unverified.'
    elif short in ce_documented:
        ce='Official table documents CE family support; this specific editor workflow was not exercised locally.'
    elif short in not_marked_ce or short.startswith('C4_MODEL_'):
        ce='Shared installed constant; inspected edition table does not mark this toolset for CE; runtime entitlement unverified.'
    else:
        ce='Shared installed constant only; native/online availability and CE entitlement not established.'
    rows.append({'constant':e['constant'],'family':family,'our_current_state':'Unbuilt plan; zero implemented coverage','our_v01_coverage':coverage,'later_milestone':milestone,'key_semantics':semantics,'deliberate_scope_loss':gap,'relative_extension_cost':cost,'evidence':'Installed API constant; selected signatures and UI-STUDY.md where applicable','ce_evidence_and_uncertainty':ce})
out=ROOT/'diagram-coverage.csv'
with out.open('w',encoding='utf-8-sig',newline='') as f:
    writer=csv.DictWriter(f,fieldnames=list(rows[0]))
    writer.writeheader();writer.writerows(rows)
assert len(rows)==101 and len({r['constant'] for r in rows})==101
print(json.dumps({'rows':len(rows),'families':dict(Counter(r['family'] for r in rows)),'milestones':dict(Counter(r['later_milestone'] for r in rows)),'planned_partial_v01':sum(r['later_milestone']=='M2-M3 / v0.1' for r in rows),'implemented':0},indent=2))
