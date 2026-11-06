# Slice S4: member text and member commands (feat/p2-member-core)

Base: `main` at `ff0c979` (S10 and B1 merged). Baseline: 294 tests pass; maintainability shows 0 FAIL and 7 WARN. Roadmap: contributes the core half of R2-04, with no status change. All of it is in `diagram-core`. The window does not change: the display lines in `ui/read_model.rs` stay as they are.

## Why

The inspector (S6) will let people type a member as one line, as most UML tools do: `- total: Money [0..1]` or `place(id: OrderId, quantity: Int): Order`. Today the core cannot do any of this:
- it can add only an attribute or an operation without parameters;
- it cannot edit a member;
- it has no way to give an operation parameters.

## Scope

| File | Work |
|---|---|
| `diagram-core/src/member_text.rs` *(new)* | `parse(text) -> Result<MemberDraft, MemberTextError>`, where `MemberDraft` is an attribute or an operation without an ID. `edit_text(&Member) -> Option<String>` gives the line to edit, or `None` when the member cannot be written as one line (see "What text can represent"); S6 then edits that member with separate fields. |
| `diagram-core/src/lib.rs` | `pub mod member_text;` and a line in the crate doc. |
| `diagram-core/src/command.rs` | `Command::AddMember { element, text }` appends the parsed member with a new ID. `Command::UpdateMember { element, member, text }` replaces the content of `member` of `element` in place, keeping its ID, its position and its kind. Labels: "Add member" and "Edit member". |
| `diagram-core/src/validate.rs` | `DomainError::MemberText(MemberTextError)` and `DomainError::MemberKindChange { from: MemberKind, to: MemberKind }` ("an attribute cannot become an operation by editing; add an operation instead", and the reverse). |
| `diagram-core/src/model.rs` | Correct the `next_id` doc: an ID is never given to two records at once, but undoing a creation also gives its ID back (see "IDs after undo"). `MemberKind` (Attribute, Operation) with words for messages. |

The display formatter in `ui/read_model.rs` is unchanged. Display and edit are two forms: the display may be lossy (it omits `+`, empty types and `Some("")` return types), while the edit form must read back exactly.

## The grammar (one member, one line)

The text is trimmed. Spaces around every symbol are optional.

1. **One line only:** a newline or other control character anywhere is refused.
2. **Visibility:** if the first character is `+`, `-`, `#` or `~`, it is the visibility (public, private, protected, package) and is removed. Otherwise the member is public, the same rule the display uses.
3. **Name:** everything up to the first `:`, `(` or `[`, trimmed.
   - It must not be empty.
   - It must not contain `) ] , < > =` or a control character. It may contain any other characters, including Arabic and inner spaces, which the core's `Name` allows.
   - The character that ends the name decides the kind: `(` makes an operation; `:`, `[` or the end of the text make an attribute. So `callback: Fn(Int)` is an attribute whose type is `Fn(Int)`.
4. **Brackets** in types and parameter lists are matched with a stack, so mixed nesting must close in order: `([)]` is refused, naming the bracket. The pairs are `()`, `[]` and `<>`. A `>` directly after `-` is part of an arrow (`->`), not a closing angle bracket, so `Fn(Int) -> Int` is a type.
5. **Operation:** `name ( parameters ) [: return type]`.
   - The parameter list ends at the `(`'s matching `)`.
   - Parameters are split at commas outside any bracket, so `Map<K, V>` stays one type.
   - Each parameter is `name` or `name : Type`, with the same name rule as above.
   - `()` means no parameters. An empty parameter between commas is refused.
   - After the `)` only `: Type` (not empty) or nothing may follow.
6. **Attribute:** `name [: Type] [multiplicity]`.
   - A multiplicity is a final `[...]` group whose `[` is either directly after the name or after whitespace. Inside it, the core's `Multiplicity::parse` applies.
   - A final `[...]` glued to a type (`list[int]`) is part of the type.
   - A `:` must be followed by a type.
7. **Default values:** a top-level `=` anywhere is refused ("default values are not supported yet"). Nothing typed is silently changed or dropped.
8. **Types:** all types (attribute, parameter, return) are trimmed, single-line and bracket-balanced.

## What text can represent

`edit_text(m)` writes the canonical line for `m`, parses it again and returns it only if the result equals `m`'s content. Otherwise it returns `None`. So the round trip is exact **by construction**, and there is no separate predicate that could drift from the parser.

**The canonical line:**
- The visibility symbol is written when the member is not public, and also when a public member's name starts with `+`, `-`, `#` or `~` (`+ -field`), so that the name is not read as a visibility.
- Then the name.
- An attribute adds `: Type` when the type is not empty, and ` [multiplicity]` when it has one.
- An operation adds `(p: T, q)` and `: Return` when there is a return type.

**Examples of what does not come back** (`edit_text` gives `None`, and S6 uses separate fields):
- a name with a `:`, a forbidden character, surrounding spaces or a newline;
- a whitespace-only or non-trimmed type;
- an attribute type ending in a whitespace-separated `[...]` group with no multiplicity after it (`T [3]`);
- a parameter type with a top-level comma (`A, B` would read back as two parameters);
- a type with a top-level `=`;
- a `Some("")` return type.

**Examples that do come back:**
- `x: T [3] [1]`: type `T [3]`, multiplicity 1;
- `+ -field`: a public name starting with `-`.

Members created through `AddMember` and `UpdateMember` always come back, because they come from `parse`: a test checks `edit_text` over many parsed lines.

## Command semantics

- **`UpdateMember`:**
  - produces exactly one element patch and allocates no ID;
  - leaves the member's ID and position, and the other members, unchanged;
  - undo and redo restore the exact before and after documents.
- **No change:** typing text that parses to the same content (including differently spaced text) is `NoChange`.
- **Refusals change nothing:** a refused command or `NoChange` leaves the whole session as it was (document, revision, saved state and redo history), because `Session::execute` returns before touching them. A test proves this.
- **Errors:**
  - a member ID that belongs to another element, or does not exist, is `MissingMember`;
  - a missing element is `MissingElement`;
  - a kind change is refused in both directions.

## IDs after undo

Undoing a creation restores the ID counter, so create → undo → create hands the same ID out again. In the core this is safe:
- no two live records ever share an ID;
- `check_document` stays clean;
- undo and redo restore every state.

But it means "a stale ID fails" holds only for IDs that are **absent** from the document. A handle kept from before an undo can name a different, newer record. So:
- the core claims only the absent-ID rule, and corrects its `next_id` comment;
- the window must fence delayed edits by document revision. S3's drafts already drop on Undo and Redo. S6's member drafts will carry the revision they were opened at and refuse to commit against a newer one. This is added to S6 in BUILD-PROGRAM.md.

The window-level create → undo → create test belongs to S5 (Class tool).

## Rules this must keep

- Core stays free of egui and SQL. The parser is a single linear pass with a bracket stack, with no backtracking.
- No new storage schema: parameters are already stored (S10). A test saves and loads an operation with parameters added and edited through text.

## Tests (written first)

- `diagram-core/tests/member_text.rs`:
  - valid attributes and operations, with and without types, multiplicities and return types;
  - each visibility prefix;
  - generics with commas; `Fn(Int) -> Int`; `std::String`; `list[int]` versus `x [1]`;
  - Arabic and mixed names; inner spaces in names;
  - every refusal case, with its message: newline, blank, missing name, crossed or unclosed brackets, text after the parameter list, an empty parameter, a `:` with no type, an invalid multiplicity, a default value;
  - the round trip over parsed lines, including `x: T [3] [1]`;
  - `edit_text` is `None` for members that cannot come back:
    - a name `"a: T"`, or with surrounding spaces, or with a newline;
    - a whitespace-only type;
    - a type `"T [3]"` with no multiplicity;
    - a parameter type `"A, B"`;
    - a type with `=`;
    - a `Some("")` return type;
  - a public name `- x` gives `+ - x`, and that reads back.
- `diagram-core/tests/commands.rs`:
  - `AddMember` adds an operation with parameters; undo removes it and redo brings it back;
  - `UpdateMember` edits in place, keeping the ID and position, with one element patch and no `NextId` change; undo and redo are exact;
  - a kind change is refused in both directions;
  - the same text, or the same text spaced differently, is `NoChange`;
  - a member ID that belongs to another class, a missing member and a missing element all fail;
  - a refused command and a `NoChange` leave the revision, saved state and redo history unchanged;
  - create → undo → create keeps live IDs unique and history exact.
- `diagram-storage/tests/load.rs`: a member added and edited through text round-trips through save and load.
- Proven to fail without the fix:
  - the comma split outside brackets;
  - the stack-based bracket matching (a single counter would accept `([)]`);
  - the attribute-versus-operation choice by the character after the name;
  - the refusal of a kind change;
  - the in-place position;
  - the `edit_text` refusal of a non-representable name.

## Gates

fmt, clippy `-D warnings`, test and doc from `app/`; maintainability at 0 FAIL with WARN not above 7; the roadmap `--check` (no status change).
- `command.rs` is already over 500 lines. The new command code stays small, and the parsing lives in `member_text.rs`, below 500 lines.
- There is no UI change and `ui/read_model.rs` is untouched, so no real-window check is needed. The UI progress page (`app/docs/ui/README.md`) gets an S4 line saying the core supports member text and that the inspector comes in S6.
