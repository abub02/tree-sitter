use super::helpers::fixtures::get_language;
use tree_sitter_tags::{TagKind, TagsConfiguration, TagsContext};

#[test]
fn test_tags_python() {
    let language = get_language("python");
    let tags_config = TagsConfiguration::new(
        language,
        r#"
        ((function_definition
          name: (identifier) @name
          body: (block . (expression_statement (string) @doc))) @function
         (strip! @doc "(^['\"\\s]*)|(['\"\\s]*$)"))
        (function_definition
          name: (identifier) @name) @function
        ((class_definition
          name: (identifier) @name
          body: (block . (expression_statement (string) @doc))) @class
         (strip! @doc "(^['\"\\s]*)|(['\"\\s]*$)"))
        (class_definition
          name: (identifier) @name) @class
        (call
          function: (identifier) @name) @call
        "#,
        "",
    )
    .unwrap();

    let source = br#"
    class Customer:
        """
        Data about a customer
        """

        def age(self):
            '''
            Get the customer's age
            '''
            compute_age(self.id)
    }
    "#;

    let mut tag_context = TagsContext::new();
    let tags = tag_context
        .generate_tags(&tags_config, source)
        .collect::<Vec<_>>();

    assert_eq!(
        tags.iter()
            .map(|t| (substr(source, &t.name_range), t.kind))
            .collect::<Vec<_>>(),
        &[
            ("Customer", TagKind::Class),
            ("age", TagKind::Function),
            ("compute_age", TagKind::Call),
        ]
    );

    assert_eq!(substr(source, &tags[0].line_range), "    class Customer:");
    assert_eq!(
        substr(source, &tags[1].line_range),
        "        def age(self):"
    );
    assert_eq!(tags[0].docs.as_ref().unwrap(), "Data about a customer");
    assert_eq!(tags[1].docs.as_ref().unwrap(), "Get the customer's age");
}

#[test]
fn test_tags_javascript() {
    let language = get_language("javascript");
    let tags_config = TagsConfiguration::new(
        language,
        r#"
        ((*
          (comment)+ @doc
          .
          (class_declaration
            name: (identifier) @name) @class)
         (select-adjacent! @doc @class)
         (strip! @doc "(^[/\\*\\s]*)|([/\\*\\s]*$)"))
        ((*
          (comment)+ @doc
          .
          (method_definition
            name: (property_identifier) @name) @method)
;         (select-adjacent! @doc @method)
         (strip! @doc "(^[/\\*\\s]*)|([/\\*\\s]*$)"))
        "#,
        "",
    )
    .unwrap();

    let mut tag_context = TagsContext::new();
    let source = br#"
    // hi

    // Data about a customer.
    // bla bla bla
    class Customer {
        /*
         * Get the customer's age
         */
        getAge() {
        }
    }

    // ok

    class Agent {

    }
    "#;
    let tags = tag_context
        .generate_tags(&tags_config, source)
        .collect::<Vec<_>>();

    assert_eq!(
        tags.iter()
            .map(|t| (substr(source, &t.name_range), t.kind))
            .collect::<Vec<_>>(),
        &[
            ("getAge", TagKind::Method),
            ("Customer", TagKind::Class),
            ("Agent", TagKind::Class)
        ]
    );
    assert_eq!(tags[0].docs.as_ref().unwrap(), "Get the customer's age");
    assert_eq!(
        tags[1].docs.as_ref().unwrap(),
        "Data about a customer.\nbla bla bla"
    );
    assert_eq!(tags[2].docs, None);
}

fn substr<'a>(source: &'a [u8], range: &std::ops::Range<usize>) -> &'a str {
    std::str::from_utf8(&source[range.clone()]).unwrap()
}
