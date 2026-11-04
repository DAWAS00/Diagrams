# Visual Paradigm versus the proposed Rust version

Date: 2026-11-04. This is a product-wide comparison based on installation inspection, public Java APIs, controlled UI experiments, native/XML project data, current official documentation, and our Rust implementation plan.

**The Rust application is not implemented.** Proposed advantages are design goals, not measured results. This analysis is complete as an initial product/architecture gap assessment; it is not a claim that every VP workflow or private algorithm has been recovered.

## Main conclusion

Visual Paradigm is a broad model-based engineering suite. Our proposed first release is a focused offline class-modeling application. We keep the essential model/view architecture and redesign access to common actions. We intentionally postpone most notation families and suite tools.

The biggest change is therefore **product scope**, followed by **interaction design** and **implementation technology**. Replacing Java with Rust does not by itself deliver feature parity, faster performance, smaller distribution or better reliability.

## Comparison documents

- [Feature differences](FEATURE-DIFFERENCES.md): core workflows, semantic behavior, export, architecture and release implications.
- [Interactive comparison](COMPARISON.html): searchable feature decisions and the complete notation inventory.
- [Diagram coverage](DIAGRAM-COVERAGE.md): all 101 installed constants, 17 analytical families and milestone mapping.
- [Coverage CSV](diagram-coverage.csv): exhaustive row-level notation inventory for tracking.
- [Broader product](BROADER-PRODUCT.md): reporting, publishing, engineering, extensions, collaboration, services and localization.
- [Validation gaps](VALIDATION-GAPS.md): uncertainty, required experiments and claims we cannot make yet.

## What we retain

Stable semantic identity; separate diagram views; typed members and relations; attached connectors; reversible transactions; project navigation; versioned saves and an export path. Keeping these protects modeling behavior even though the UI is smaller.

## What we redesign

One-step diagram creation; inline naming/member entry; a contextual inspector; explicit **Show in diagram** versus **Duplicate model**; a compact toolbar; visible save/recovery feedback; and a clear separation between model properties and view appearance.

These changes target discoverability and fewer interruptions. The basic modeling tasks still need equivalent semantic results. A side panel must not silently remove important properties just because their dialogs were inconvenient.

## What users initially lose

Most diagram families; advanced UML properties/profiles; sophisticated layout/routing; rich copy/paste formats; existing VP-file compatibility; PNG/PDF and broader export options; document/report composition; code/DB engineering; plugin SDK/automation; cross-project references; repository collaboration; and online/AI tools. Some of these belong to paid/service-backed VP products rather than confirmed CE functionality.

The first release is not a replacement for users relying on those capabilities. It is useful for the explicitly defined class-modeling workflow.

## New evidence in this comparison

1. REA 6.7.0 is connected. Its SQLite inspector examined the saved research project and confirmed its schema and three original diagram rows. Preserved evidence: `research/visual-paradigm/ce-evidence/rea-sqlite-evidence.json`, ID `ev_3c3f507d4c5134d87ca76e26fc08d3a92976bf94e72430409bbb3b72867d03dc`.
2. In the running CE application, a selected class offered internal/image/XML copy variants and **Copy to Diagram**. That dialog could create a new diagram or use an existing one.
3. Copying Customer into a new class diagram produced an auxiliary view. The separately saved comparison project contains **two Class models and three Class views**; two distinct diagram views reference the same Customer model ID.
4. The comparison fixture has four diagrams and passes a separate Python SQLite quick check. See `research/visual-paradigm/ce-evidence/comparison-project-inspection.json` and `comparison-research.vpp`.

Cross-view rename and independent styling were not executed in this pass. The shared IDs establish reuse, not every update-propagation rule. The original disposable fixture's hash changed across resumed sessions, so the reports preserve acquisition hashes and do not describe that source as an immutable baseline.

Ghidra is not configured in the connected REA server. No native decompilation was performed. Because VP's diagram functionality is in its Java platform, decompiling its Windows launcher would not by itself explain the diagram engine anyway.

## Recommended decision

Keep the agreed Rust/egui prototype gate and class vertical slice. Add no suite features simply to match visible buttons. Before expanding the release, prioritize user outcomes: usable class members/relationships, model reuse, clear navigation, save/reopen, export, keyboard/text/DPI behavior and failure recovery.

Then validate Flowchart/Use Case adapters, ER and sequence as separate milestones. Treat PDF/PNG, VP XML interoperability and cross-project work as explicit product decisions, because they may matter more to real users than a large palette.
