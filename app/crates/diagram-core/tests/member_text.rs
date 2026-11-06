//! R2-04 (core half): one member typed as one line. Parsing, refusals with
//! messages in words, and the exact round trip of `edit_text`.

use diagram_core::ids::MemberId;
use diagram_core::member_text::{self, MemberDraft, MemberTextError};
use diagram_core::model::{
    Attribute, Member, Multiplicity, MultiplicityError, Name, Operation, Parameter, Visibility,
};

fn name(text: &str) -> Name {
    Name::new(text).unwrap_or_else(|_| name("fallback"))
}

fn multiplicity(text: &str) -> Option<Multiplicity> {
    Multiplicity::parse(text).ok()
}

fn attribute(
    visibility: Visibility,
    text: &str,
    type_text: &str,
    many: Option<&str>,
) -> MemberDraft {
    MemberDraft::Attribute {
        visibility,
        name: name(text),
        type_text: type_text.to_owned(),
        multiplicity: many.and_then(multiplicity),
    }
}

fn parameter(text: &str, type_text: &str) -> Parameter {
    Parameter {
        name: name(text),
        type_text: type_text.to_owned(),
    }
}

fn operation(
    visibility: Visibility,
    text: &str,
    parameters: Vec<Parameter>,
    return_type: Option<&str>,
) -> MemberDraft {
    MemberDraft::Operation {
        visibility,
        name: name(text),
        parameters,
        return_type: return_type.map(str::to_owned),
    }
}

fn parsed(text: &str) -> MemberDraft {
    let result = member_text::parse(text);
    assert!(result.is_ok(), "{text:?} should parse: {result:?}");
    result.unwrap_or_else(|_| attribute(Visibility::Public, "unreachable", "", None))
}

fn refused(text: &str) -> MemberTextError {
    let result = member_text::parse(text);
    assert!(
        result.is_err(),
        "{text:?} should be refused, got {result:?}"
    );
    result.err().unwrap_or(MemberTextError::Blank)
}

use Visibility::{Package, Private, Protected, Public};

#[test]
fn attributes_should_parse_with_optional_type_and_multiplicity() {
    assert_eq!(parsed("name"), attribute(Public, "name", "", None));
    assert_eq!(
        parsed("name: String"),
        attribute(Public, "name", "String", None)
    );
    assert_eq!(
        parsed("  name :String  "),
        attribute(Public, "name", "String", None)
    );
    assert_eq!(
        parsed("- total: Money [0..1]"),
        attribute(Private, "total", "Money", Some("0..1"))
    );
    assert_eq!(parsed("tags [*]"), attribute(Public, "tags", "", Some("*")));
    assert_eq!(
        parsed("tags[1..*]"),
        attribute(Public, "tags", "", Some("1..*"))
    );
}

#[test]
fn each_visibility_prefix_should_be_read() {
    assert_eq!(parsed("+ a"), attribute(Public, "a", "", None));
    assert_eq!(parsed("-a"), attribute(Private, "a", "", None));
    assert_eq!(parsed("# a: T"), attribute(Protected, "a", "T", None));
    assert_eq!(parsed("~a()"), operation(Package, "a", vec![], None));
}

#[test]
fn operations_should_parse_parameters_and_return_type() {
    assert_eq!(parsed("ping()"), operation(Public, "ping", vec![], None));
    assert_eq!(
        parsed("place(id: OrderId, quantity: Int): Order"),
        operation(
            Public,
            "place",
            vec![parameter("id", "OrderId"), parameter("quantity", "Int")],
            Some("Order")
        )
    );
    assert_eq!(
        parsed("# find( id , flag :Bool ) : Order"),
        operation(
            Protected,
            "find",
            vec![parameter("id", ""), parameter("flag", "Bool")],
            Some("Order")
        )
    );
}

#[test]
fn commas_and_colons_inside_types_should_not_split() {
    assert_eq!(
        parsed("index(map: Map<K, V>, path: std::Path): Map<K, List<V>>"),
        operation(
            Public,
            "index",
            vec![
                parameter("map", "Map<K, V>"),
                parameter("path", "std::Path")
            ],
            Some("Map<K, List<V>>")
        )
    );
    assert_eq!(
        parsed("cache: Map<String, (Int, Int)>"),
        attribute(Public, "cache", "Map<String, (Int, Int)>", None)
    );
}

#[test]
fn the_character_after_the_name_should_decide_the_kind() {
    // A function type is an attribute, not an operation.
    assert_eq!(
        parsed("callback: Fn(Int) -> Int"),
        attribute(Public, "callback", "Fn(Int) -> Int", None)
    );
    assert_eq!(
        parsed("apply(f: Fn(Int) -> Int): Int"),
        operation(
            Public,
            "apply",
            vec![parameter("f", "Fn(Int) -> Int")],
            Some("Int")
        )
    );
}

#[test]
fn a_bracket_glued_to_a_type_should_stay_in_the_type() {
    assert_eq!(
        parsed("ids: list[int]"),
        attribute(Public, "ids", "list[int]", None)
    );
    assert_eq!(
        parsed("ids: list[int] [0..1]"),
        attribute(Public, "ids", "list[int]", Some("0..1"))
    );
}

#[test]
fn arabic_and_mixed_names_should_parse() {
    assert_eq!(parsed("- اسم: نص"), attribute(Private, "اسم", "نص", None));
    assert_eq!(
        parsed("احسب(مبلغ: Money): Money"),
        operation(
            Public,
            "احسب",
            vec![parameter("مبلغ", "Money")],
            Some("Money")
        )
    );
    assert_eq!(
        parsed("order date: Date"),
        attribute(Public, "order date", "Date", None)
    );
}

#[test]
fn every_malformed_line_should_be_refused_with_a_reason() {
    let cases: [(&str, MemberTextError); 20] = [
        ("", MemberTextError::Blank),
        ("   ", MemberTextError::Blank),
        ("a\nb", MemberTextError::MultipleLines),
        ("- ", MemberTextError::MissingName),
        (": Int", MemberTextError::MissingName),
        ("(): Int", MemberTextError::MissingName),
        ("a: ", MemberTextError::MissingType),
        ("a(): ", MemberTextError::MissingType),
        ("a(b: ): Int", MemberTextError::MissingType),
        ("a(b,,c)", MemberTextError::EmptyParameter(2)),
        ("a(b) c", MemberTextError::TextAfterParameters),
        ("a(b", MemberTextError::Unclosed('(')),
        ("a: Map<K, V", MemberTextError::Unclosed('<')),
        (
            "a: (Int]",
            MemberTextError::Mismatched {
                open: '(',
                close: ']',
            },
        ),
        ("a: Int)", MemberTextError::Unopened(')')),
        ("count: Int = 0", MemberTextError::DefaultValue),
        ("f() = 1", MemberTextError::DefaultValue),
        ("a [1] = 1", MemberTextError::DefaultValue),
        ("f(): Int = 1", MemberTextError::DefaultValue),
        ("f(a: Int = 1)", MemberTextError::DefaultValue),
    ];
    for (text, expected) in cases {
        assert_eq!(refused(text), expected, "{text:?}");
        assert!(!expected.to_string().is_empty());
    }
}

#[test]
fn a_control_character_at_either_end_should_be_refused_not_trimmed() {
    for text in [
        "a\u{2028}b",
        "name\u{2029}",
        "f(a: T\u{2028})",
        "a: Map<K,\u{2028}V>",
        "name\n",
        "\nname",
        "name\t",
        "\tname: Int",
        "name\r\n",
    ] {
        assert_eq!(refused(text), MemberTextError::MultipleLines, "{text:?}");
    }
}

#[test]
fn crossed_brackets_should_be_refused() {
    assert_eq!(
        refused("a: ([)]"),
        MemberTextError::Mismatched {
            open: '[',
            close: ')'
        }
    );
}

#[test]
fn forbidden_characters_in_names_and_bad_multiplicities_should_be_refused() {
    assert_eq!(refused("a,b: Int"), MemberTextError::NameCharacter(','));
    assert_eq!(refused("a>b"), MemberTextError::NameCharacter('>'));
    assert_eq!(refused("f(a<b>: T)"), MemberTextError::NameCharacter('<'));
    assert!(matches!(
        refused("a: Int [5..2]"),
        MemberTextError::Multiplicity(MultiplicityError::Inverted { .. })
    ));
    assert_eq!(refused("a [1] b"), MemberTextError::TextAfterMultiplicity);
}

#[test]
fn messages_should_be_words() {
    assert_eq!(
        MemberTextError::DefaultValue.to_string(),
        "default values (\"= …\") are not supported yet"
    );
    assert_eq!(
        MemberTextError::EmptyParameter(2).to_string(),
        "parameter 2 is empty"
    );
}

fn as_member(draft: MemberDraft) -> Member {
    draft.into_member(MemberId::from_raw(7))
}

#[test]
fn edit_text_should_read_back_exactly() {
    let members = [
        "name",
        "- total: Money [0..1]",
        "# find(id: OrderId, flag): Order",
        "~ index(map: Map<K, V>): List<V>",
        "callback: Fn(Int) -> Int",
        "ids: list[int] [*]",
        "x: T [3] [1]",
        "+ -field",
        "- اسم: نص",
        "ping()",
    ];
    for text in members {
        let member = as_member(parsed(text));
        let line = member_text::edit_text(&member);
        assert!(line.is_some(), "{text:?} is representable");
        let read_back = line.as_deref().map(|line| as_member(parsed(line)));
        assert_eq!(read_back, Some(member), "{text:?} -> {line:?}");
    }
}

#[test]
fn members_text_cannot_represent_should_give_no_edit_text() {
    let attribute_with = |name_text: &str, type_text: &str| {
        Member::Attribute(Attribute {
            id: MemberId::from_raw(1),
            name: name(name_text),
            type_text: type_text.to_owned(),
            visibility: Public,
            multiplicity: None,
        })
    };
    let unrepresentable = [
        attribute_with("a: T", ""),
        attribute_with(" a", "T"),
        attribute_with("a\nb", ""),
        attribute_with("a", "   "),
        attribute_with("a", "T [3]"),
        attribute_with("a", "T = 1"),
        Member::Operation(Operation {
            id: MemberId::from_raw(2),
            name: name("f"),
            visibility: Public,
            parameters: vec![],
            return_type: Some(String::new()),
        }),
        Member::Operation(Operation {
            id: MemberId::from_raw(3),
            name: name("f"),
            visibility: Public,
            parameters: vec![Parameter {
                name: name("p"),
                type_text: String::from("A, B"),
            }],
            return_type: None,
        }),
    ];
    for member in unrepresentable {
        assert_eq!(member_text::edit_text(&member), None, "{member:?}");
    }
}

#[test]
fn a_public_name_starting_with_a_visibility_mark_should_get_an_explicit_plus() {
    let member = Member::Attribute(Attribute {
        id: MemberId::from_raw(7),
        name: name("- x"),
        type_text: String::new(),
        visibility: Public,
        multiplicity: None,
    });

    let line = member_text::edit_text(&member);

    assert_eq!(line.as_deref(), Some("+ - x"));
    assert_eq!(line.map(|line| as_member(parsed(&line))), Some(member));
}

#[test]
fn drafts_should_report_their_kind() {
    use diagram_core::model::MemberKind;
    assert_eq!(parsed("a").kind(), MemberKind::Attribute);
    assert_eq!(parsed("a()").kind(), MemberKind::Operation);
    assert_eq!(
        as_member(parsed("a()")).kind(),
        MemberKind::Operation,
        "a member reports the same kind"
    );
}
