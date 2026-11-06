//! One member typed as one line, such as `- total: Money [0..1]` or
//! `place(id: OrderId, quantity: Int): Order` (R2-04).
//!
//! [`parse`] reads a line into a [`MemberDraft`]: content without an ID.
//! [`edit_text`] writes the line to edit an existing member. It checks
//! itself by parsing the line again and gives `None` when the member would
//! not come back exactly (for example a name containing `:`), so an edit
//! never changes what the user did not touch. The display lines on the
//! canvas are a separate, lossy form owned by the window.
//!
//! The grammar, in order:
//! 1. One line: no newline or control character.
//! 2. An optional visibility symbol `+ - # ~` first; public otherwise.
//! 3. The name runs to the first `:`, `(` or `[`. That character decides
//!    the kind: `(` makes an operation, anything else an attribute.
//! 4. Brackets `()`, `[]` and `<>` must close in order (a stack, not a
//!    counter). A `>` right after `-` is part of an arrow `->`.
//! 5. Operation: `name(p: T, q): Return`. Parameters split at commas
//!    outside brackets.
//! 6. Attribute: `name: Type [multiplicity]`. A final `[...]` is a
//!    multiplicity when its `[` follows the name or whitespace; glued to a
//!    type (`list[int]`) it belongs to the type.
//! 7. A top-level `=` (a default value) is refused, not kept or dropped.

use thiserror::Error;

use crate::ids::MemberId;
use crate::model::{
    Attribute, Member, MemberKind, Multiplicity, MultiplicityError, Name, Operation, Parameter,
    Visibility,
};

/// Why a line is not a member.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MemberTextError {
    /// Nothing was typed.
    #[error("type a member, such as \"name: Type\" or \"name(): Type\"")]
    Blank,
    /// The text has a line break or another control character.
    #[error("a member is one line")]
    MultipleLines,
    /// There is no name before `:`, `(` or `[`.
    #[error("the member needs a name")]
    MissingName,
    /// A name contains a character that has a meaning in the line.
    #[error("a name cannot contain \"{0}\"")]
    NameCharacter(char),
    /// A `:` with nothing after it.
    #[error("a \":\" must be followed by a type")]
    MissingType,
    /// An opening bracket is never closed.
    #[error("\"{0}\" is not closed")]
    Unclosed(char),
    /// A closing bracket with no opening one.
    #[error("\"{0}\" has no opening bracket")]
    Unopened(char),
    /// Brackets cross, as in `([)]`.
    #[error("\"{close}\" cannot close \"{open}\"")]
    Mismatched {
        /// The bracket still open.
        open: char,
        /// The closing bracket found.
        close: char,
    },
    /// Two commas with nothing between them (1-based position).
    #[error("parameter {0} is empty")]
    EmptyParameter(usize),
    /// Something other than `: Type` after the parameter list.
    #[error("only \": Type\" can follow the parameter list")]
    TextAfterParameters,
    /// Something after the multiplicity.
    #[error("nothing can follow the multiplicity")]
    TextAfterMultiplicity,
    /// A default value such as `= 0`.
    #[error("default values (\"= …\") are not supported yet")]
    DefaultValue,
    /// The multiplicity is invalid.
    #[error(transparent)]
    Multiplicity(#[from] MultiplicityError),
}

/// A member's content without its ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberDraft {
    /// An attribute.
    Attribute {
        /// Visibility.
        visibility: Visibility,
        /// Name.
        name: Name,
        /// Type text; empty when none was typed.
        type_text: String,
        /// Multiplicity.
        multiplicity: Option<Multiplicity>,
    },
    /// An operation.
    Operation {
        /// Visibility.
        visibility: Visibility,
        /// Name.
        name: Name,
        /// Parameters in order.
        parameters: Vec<Parameter>,
        /// Return type.
        return_type: Option<String>,
    },
}

impl MemberDraft {
    /// Attribute or operation.
    pub const fn kind(&self) -> MemberKind {
        match self {
            Self::Attribute { .. } => MemberKind::Attribute,
            Self::Operation { .. } => MemberKind::Operation,
        }
    }

    /// The member with identity `id`.
    pub fn into_member(self, id: MemberId) -> Member {
        match self {
            Self::Attribute {
                visibility,
                name,
                type_text,
                multiplicity,
            } => Member::Attribute(Attribute {
                id,
                name,
                type_text,
                visibility,
                multiplicity,
            }),
            Self::Operation {
                visibility,
                name,
                parameters,
                return_type,
            } => Member::Operation(Operation {
                id,
                name,
                visibility,
                parameters,
                return_type,
            }),
        }
    }

    /// The content of `member`.
    pub fn of(member: &Member) -> Self {
        match member.clone() {
            Member::Attribute(attribute) => Self::Attribute {
                visibility: attribute.visibility,
                name: attribute.name,
                type_text: attribute.type_text,
                multiplicity: attribute.multiplicity,
            },
            Member::Operation(operation) => Self::Operation {
                visibility: operation.visibility,
                name: operation.name,
                parameters: operation.parameters,
                return_type: operation.return_type,
            },
        }
    }
}

/// One character of a line with its bracket depth: `depth` is the depth
/// outside the character for brackets and the current depth otherwise.
struct Scanned {
    index: usize,
    character: char,
    depth: usize,
}

fn closing(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        _ => '>',
    }
}

/// Checks that brackets close in order and records each character's depth.
fn scan(text: &str) -> Result<Vec<Scanned>, MemberTextError> {
    let mut stack: Vec<char> = Vec::new();
    let mut out = Vec::with_capacity(text.len());
    let mut previous = None;
    for (index, character) in text.char_indices() {
        let arrow = character == '>' && previous == Some('-');
        let depth = match character {
            '(' | '[' | '<' => {
                stack.push(character);
                stack.len() - 1
            }
            ')' | ']' | '>' if !arrow => {
                let Some(open) = stack.pop() else {
                    return Err(MemberTextError::Unopened(character));
                };
                if closing(open) != character {
                    return Err(MemberTextError::Mismatched {
                        open,
                        close: character,
                    });
                }
                stack.len()
            }
            _ => stack.len(),
        };
        out.push(Scanned {
            index,
            character,
            depth,
        });
        previous = Some(character);
    }
    match stack.pop() {
        Some(open) => Err(MemberTextError::Unclosed(open)),
        None => Ok(out),
    }
}

/// A trimmed type that must not be empty and has no top-level `=`.
fn required_type(text: &str) -> Result<String, MemberTextError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(MemberTextError::MissingType);
    }
    if scan(text)?
        .iter()
        .any(|scanned| scanned.depth == 0 && scanned.character == '=')
    {
        return Err(MemberTextError::DefaultValue);
    }
    Ok(text.to_owned())
}

/// A trimmed name without characters that have a meaning in the line.
fn checked_name(text: &str) -> Result<Name, MemberTextError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(MemberTextError::MissingName);
    }
    if text.contains('=') {
        return Err(MemberTextError::DefaultValue);
    }
    if let Some(character) = text
        .chars()
        .find(|character| ":()[],<>".contains(*character))
    {
        return Err(MemberTextError::NameCharacter(character));
    }
    Name::new(text).map_err(|_| MemberTextError::MissingName)
}

/// Splits `name : Type` at its first top-level `:`; the type is optional.
fn name_and_type(text: &str) -> Result<(Name, String), MemberTextError> {
    let colon = scan(text)?
        .into_iter()
        .find(|scanned| scanned.depth == 0 && scanned.character == ':')
        .map(|scanned| scanned.index);
    match colon {
        Some(colon) => Ok((
            checked_name(&text[..colon])?,
            required_type(&text[colon + 1..])?,
        )),
        None => Ok((checked_name(text)?, String::new())),
    }
}

fn parameters(text: &str) -> Result<Vec<Parameter>, MemberTextError> {
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut pieces = Vec::new();
    let mut start = 0;
    for scanned in scan(text)? {
        if scanned.depth == 0 && scanned.character == ',' {
            pieces.push(&text[start..scanned.index]);
            start = scanned.index + 1;
        }
    }
    pieces.push(&text[start..]);
    pieces
        .into_iter()
        .enumerate()
        .map(|(position, piece)| {
            if piece.trim().is_empty() {
                return Err(MemberTextError::EmptyParameter(position + 1));
            }
            let (name, type_text) = name_and_type(piece)?;
            Ok(Parameter { name, type_text })
        })
        .collect()
}

fn operation(
    visibility: Visibility,
    name: Name,
    rest: &str,
) -> Result<MemberDraft, MemberTextError> {
    // `rest` starts with the `(`; its partner is the first `)` back at depth 0.
    let close = scan(rest)?
        .into_iter()
        .find(|scanned| scanned.depth == 0 && scanned.character == ')')
        .map_or(rest.len(), |scanned| scanned.index);
    let inside = rest.get(1..close).unwrap_or_default();
    let after = rest.get(close + 1..).unwrap_or_default().trim();
    let return_type = match after.strip_prefix(':') {
        Some(type_text) => Some(required_type(type_text)?),
        None if after.is_empty() => None,
        None => return Err(MemberTextError::TextAfterParameters),
    };
    Ok(MemberDraft::Operation {
        visibility,
        name,
        parameters: parameters(inside)?,
        return_type,
    })
}

/// The `[...]` group that ends `text`, if any: (start of `[`, inside).
fn final_group(text: &str) -> Result<Option<(usize, &str)>, MemberTextError> {
    if !text.ends_with(']') {
        return Ok(None);
    }
    let open = scan(text)?
        .into_iter()
        .rev()
        .find(|scanned| scanned.depth == 0 && scanned.character == '[')
        .map(|scanned| scanned.index);
    Ok(open.map(|open| (open, &text[open + 1..text.len() - 1])))
}

fn attribute(
    visibility: Visibility,
    name: Name,
    rest: &str,
) -> Result<MemberDraft, MemberTextError> {
    let rest = rest.trim();
    let (type_text, multiplicity) = if let Some(type_part) = rest.strip_prefix(':') {
        let type_part = type_part.trim();
        match final_group(type_part)? {
            Some((open, inside))
                if type_part[..open]
                    .chars()
                    .last()
                    .is_some_and(char::is_whitespace) =>
            {
                (
                    required_type(&type_part[..open])?,
                    Some(Multiplicity::parse(inside)?),
                )
            }
            _ => (required_type(type_part)?, None),
        }
    } else if rest.is_empty() {
        (String::new(), None)
    } else {
        // `rest` starts with `[`: exactly one group, nothing after it.
        match final_group(rest)? {
            Some((0, inside)) => (String::new(), Some(Multiplicity::parse(inside)?)),
            _ => return Err(MemberTextError::TextAfterMultiplicity),
        }
    };
    Ok(MemberDraft::Attribute {
        visibility,
        name,
        type_text,
        multiplicity,
    })
}

/// Reads one member from `text`.
///
/// # Errors
///
/// Returns the [`MemberTextError`] that says, in words, what to change.
pub fn parse(text: &str) -> Result<MemberDraft, MemberTextError> {
    if text.trim().is_empty() {
        return Err(MemberTextError::Blank);
    }
    // Checked before trimming, so a newline or tab at either end is refused
    // too instead of being silently removed.
    if text.chars().any(char::is_control) {
        return Err(MemberTextError::MultipleLines);
    }
    let text = text.trim();
    let (visibility, rest) = match text.chars().next() {
        Some('+') => (Visibility::Public, &text[1..]),
        Some('-') => (Visibility::Private, &text[1..]),
        Some('#') => (Visibility::Protected, &text[1..]),
        Some('~') => (Visibility::Package, &text[1..]),
        _ => (Visibility::Public, text),
    };
    let rest = rest.trim_start();
    let name_end = rest.find([':', '(', '[']).unwrap_or(rest.len());
    let name = checked_name(&rest[..name_end])?;
    // Brackets are checked over the whole remainder before it is split.
    scan(&rest[name_end..])?;
    if rest[name_end..].starts_with('(') {
        operation(visibility, name, &rest[name_end..])
    } else {
        attribute(visibility, name, &rest[name_end..])
    }
}

/// The canonical line for `member`, before checking that it reads back.
fn canonical(member: &Member) -> String {
    let (visibility, name) = match member {
        Member::Attribute(attribute) => (attribute.visibility, &attribute.name),
        Member::Operation(operation) => (operation.visibility, &operation.name),
    };
    let needs_symbol =
        visibility != Visibility::Public || name.as_str().starts_with(['+', '-', '#', '~']);
    let mut line = if needs_symbol {
        format!("{} {name}", visibility.symbol())
    } else {
        name.to_string()
    };
    match member {
        Member::Attribute(attribute) => {
            if !attribute.type_text.is_empty() {
                line.push_str(": ");
                line.push_str(&attribute.type_text);
            }
            if let Some(multiplicity) = attribute.multiplicity {
                line.push_str(&format!(" [{multiplicity}]"));
            }
        }
        Member::Operation(operation) => {
            let parameters: Vec<String> = operation
                .parameters
                .iter()
                .map(|parameter| {
                    if parameter.type_text.is_empty() {
                        parameter.name.to_string()
                    } else {
                        format!("{}: {}", parameter.name, parameter.type_text)
                    }
                })
                .collect();
            line.push_str(&format!("({})", parameters.join(", ")));
            if let Some(return_type) = &operation.return_type {
                line.push_str(": ");
                line.push_str(return_type);
            }
        }
    }
    line
}

/// The line to edit `member`, or `None` when no line would read back as
/// exactly this member; the member is then edited field by field.
pub fn edit_text(member: &Member) -> Option<String> {
    let line = canonical(member);
    (parse(&line).ok()? == MemberDraft::of(member)).then_some(line)
}
