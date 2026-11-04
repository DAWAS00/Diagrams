# 03 — A simpler GUI

## Interaction direction

Build around the current document and selection. Use a compact toolbar instead of exposing unrelated product areas. Keep simple edits inline and detailed fields in a persistent inspector. This is a proposed improvement to validate with users, not a measured result yet.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│ Project ▾   Undo Redo    + Add   Connect    Arrange ▾        Save  Export   │
├─────────────────┬──────────────────────────────────────────┬───────────────┤
│ Project search  │ Class diagram 1  ×   Overview  ×     +    │ Selection     │
│ Diagrams|Model  ├──────────────────────────────────────────┤ Name          │
│                 │ Select  Class  Association  Note         │ Members       │
│ Current diagram │                                          │ Relationships │
│ Other diagrams  │              Diagram canvas              │ Appearance ▾  │
│                 │                                          │ Advanced ▾    │
│ + Diagram       │                                          │               │
├─────────────────┴──────────────────────────────────────────┴───────────────┤
│ Saved / Unsaved · local path       Validation                  − 100% + Fit │
└────────────────────────────────────────────────────────────────────────────┘
```

See [GUI-CONCEPT.html](GUI-CONCEPT.html) for an interactive layout concept. Its HTML is a planning aid; the application GUI will be Rust/egui.

## Five core workflows

### Start a diagram

Welcome page: **New project**, **Open project**, recent files. New project shows a small searchable type selector with only implemented types. Choose Class and enter a project name; open a blank diagram immediately. Templates are optional and later, rather than a mandatory wizard page. Naming, diagram description and folder ownership can be edited afterward.

### Add and name

Use the labeled Class tool, drag it, press the documented shortcut outside a text field, or use canvas quick-add. Place the node and focus its name. Enter commits; Escape cancels the draft. The inspector offers Add attribute and Add operation without a nested modal chain. A compact syntax input is optional shorthand; detailed typed fields remain available.

### Connect

Hover/focus a node to reveal connection handles with sufficiently large hit regions. Drag to a valid target; preview arrowhead/route and highlight admissible targets. Choose relationship type from the current notation. Commit relation plus view as one transaction. Inspector shows readable endpoint roles and multiplicities. Invalid connection leaves the document unchanged and explains why.

### Edit and reuse

Click selects a view; its inspector displays **Model** and **Appearance** sections. Rename updates all model views; Appearance affects the selected view. **Show in diagram** creates another view of the same semantic object. **Duplicate model** creates new IDs. On Delete remove selected view by default; **Delete model everywhere…** is a separate explicit action that previews dependent views/relations before confirmation. Undo restores the complete graph.

### Save and share

Ctrl+S saves to the native file; status shows Saving, Saved or a persistent error with retry. Initial save uses a native file picker. Export chooses the active diagram and format, with canvas bounds/background options. The basic release exports SVG. Closing a dirty project offers Save, Discard or Cancel, and never closes while a save has failed unnoticed.

## Visual language

- Light theme first with dark mode as a supported preference, not the sole readability solution.
- Restrained neutral workspace, distinct white/light canvas, one action accent and a separate semantic warning color.
- Clear typography; 14–16 logical-pixel UI text as the starting design target, adjustable scaling.
- Visible text labels for primary tools; icons and hover tips supplement them.
- Consistent 8-unit spacing, 32–36-unit controls, larger invisible connection/resize hit areas.
- Keep canvas object colors independent from editor theme so exported documents stay predictable.
- Show the project path and dirty state; never use only color to signal selection, errors or save state.
- Collapse Appearance/Advanced by default. A first-time user should not need stereotypes, profiles or service/account menus to draw a class.

## Keyboard, focus and accessibility

Document Ctrl+N/O/S, Ctrl+Shift+S, Ctrl+Z and Ctrl+Shift+Z, Delete, Escape, Enter/F2, Space+drag pan, and zoom/fit controls. Only bind canvas creation keys when canvas has focus. Preserve normal copy/undo inside text inputs. Arrow keys move the selected view with clear modifiers for fine/coarse movement. Support mouse wheel and touchpad interaction without requiring a middle button.

The model/object list is a keyboard-accessible alternative to tiny canvas targets. Provide accessible names, selected state, position and relationships for canvas objects; standard egui widget accessibility alone is insufficient for a custom canvas. Test screen-reader traversal in M0/M3. Test English and Arabic names, mixed-direction text, paste and IME; failure is a GUI decision gate, not something to hide with unsupported-font warnings.

## Responsive desktop behavior

At about 1280×800, use a 220-unit project sidebar and 280-unit inspector, both resizable. Below the usable canvas width, collapse the sidebar and show the inspector as an explicit drawer; keep toolbar overflow searchable. Verify 100%, 150% and 200% Windows display scaling and moving between monitors. Avoid arbitrary dockable/floating panels in the initial release.

## Validation of “easier”

Compare the same tasks against our recorded VP workflow: initial creation, adding a member, connecting with multiplicity, reusing an element, finding an object and exporting. Record dialog count, time and recovery from mistakes. Use actual user feedback before claiming superiority. If shortcuts/context menus are undiscoverable, improve labeled controls rather than adding more hidden gestures.
