use sir_ir::{Application, Effect, Expr, FieldType, Literal};

fn app() -> Application {
    serde_json::from_str(include_str!("../../../examples/expenses.sir.json")).unwrap()
}
fn codes(app: &Application) -> Vec<&'static str> {
    sir_validator::validate(app)
        .into_iter()
        .map(|d| d.code)
        .collect()
}
#[test]
fn executable_fixture_and_entity_only_fixture_validate() {
    assert!(codes(&app()).is_empty());
    let entity_only =
        serde_json::from_str(include_str!("../../../examples/expenses.entities.json")).unwrap();
    assert!(codes(&entity_only).is_empty());
}
#[test]
fn unknown_reference_and_unknown_actor_fail() {
    let mut app = app();
    app.entities[0].fields[0].field_type = FieldType::Ref {
        entity: "Missing".into(),
    };
    app.commands[0].actor = "MissingActor".into();
    assert!(codes(&app).contains(&"UNKNOWN_ENTITY"));
}
#[test]
fn duplicate_names_and_reserved_inputs_fail() {
    let mut app = app();
    app.entities[1].name = app.entities[0].name.clone();
    app.entities[0].fields[1].name = app.entities[0].fields[0].name.clone();
    app.commands[0].inputs[0].name = "actor".into();
    assert!(codes(&app).contains(&"DUPLICATE_NAME"));
    assert!(codes(&app).contains(&"INVALID_IDENTIFIER"));
}
#[test]
fn version_empty_entities_and_bad_identifiers_fail() {
    let mut app = app();
    app.sir_version = "future".into();
    app.name = "invalid name".into();
    app.entities.clear();
    assert!(codes(&app).contains(&"UNSUPPORTED_VERSION"));
    assert!(codes(&app).contains(&"INVALID_IDENTIFIER"));
    assert!(codes(&app).contains(&"LIMIT_EXCEEDED"));
}
#[test]
fn missing_policy_unknown_keys_and_unknown_operations_fail_parsing() {
    let base = serde_json::to_value(app()).unwrap();
    let mut value = base.clone();
    value["commands"][0]
        .as_object_mut()
        .unwrap()
        .remove("policy");
    assert!(serde_json::from_value::<Application>(value).is_err());
    let mut value = base.clone();
    value["commands"][0]["policy"]["shell"] = serde_json::json!("anything");
    assert!(serde_json::from_value::<Application>(value).is_err());
    let mut value = base;
    value["commands"][0]["policy"]["op"] = serde_json::json!("eval");
    assert!(serde_json::from_value::<Application>(value).is_err());
}
#[test]
fn policy_must_be_boolean() {
    let mut app = app();
    app.commands[0].policy = Expr::Literal {
        value: Literal::String("true".into()),
    };
    assert!(codes(&app).contains(&"TYPE_MISMATCH"));
}
#[test]
fn effect_type_mismatch_and_actor_root_fail() {
    let mut app = app();
    app.commands[0].effects = vec![Effect::Set {
        target: vec!["expense".into(), "amount_minor".into()],
        value: Expr::Literal {
            value: Literal::String("wrong".into()),
        },
    }];
    assert!(codes(&app).contains(&"TYPE_MISMATCH"));
    app.commands[0].effects = vec![Effect::Set {
        target: vec!["actor".into(), "email".into()],
        value: Expr::Literal {
            value: Literal::String("changed".into()),
        },
    }];
    assert!(codes(&app).contains(&"INVALID_EFFECT"));
}
#[test]
fn unknown_binding_scalar_traversal_and_unknown_fields_fail() {
    for (segments, expected) in [
        (vec!["unknown"], "UNKNOWN_BINDING"),
        (vec!["expense", "missing"], "UNKNOWN_FIELD"),
        (vec!["expense", "status", "missing"], "INVALID_PATH"),
    ] {
        let mut app = app();
        app.commands[0].policy = Expr::Path {
            segments: segments.into_iter().map(String::from).collect(),
        };
        assert!(codes(&app).contains(&expected));
    }
}
#[test]
fn empty_effects_and_empty_boolean_expressions_fail() {
    let mut app = app();
    app.commands[0].effects.clear();
    app.commands[0].policy = Expr::And { args: vec![] };
    assert!(codes(&app).contains(&"EMPTY_COMMAND"));
    assert!(codes(&app).contains(&"LIMIT_EXCEEDED"));
}
#[test]
fn equality_cannot_compare_different_reference_types() {
    let mut app = app();
    app.commands[0].policy = Expr::Eq {
        left: Box::new(Expr::Path {
            segments: vec!["expense".into()],
        }),
        right: Box::new(Expr::Path {
            segments: vec!["actor".into()],
        }),
    };
    assert!(codes(&app).contains(&"TYPE_MISMATCH"));
}
#[test]
fn nested_expressions_and_paths_are_bounded() {
    let mut app = app();
    let mut expr = Expr::Literal {
        value: Literal::Bool(true),
    };
    for _ in 0..20 {
        expr = Expr::Not {
            arg: Box::new(expr),
        };
    }
    app.commands[0].policy = expr;
    assert!(codes(&app).contains(&"LIMIT_EXCEEDED"));
    app.commands[0].policy = Expr::Path {
        segments: vec!["actor".into(); 9],
    };
    assert!(codes(&app).contains(&"INVALID_PATH"));
}
#[test]
fn integer_literal_outside_i64_is_rejected() {
    let value = r#"{"op":"literal","value":{"kind":"int","value":9223372036854775808}}"#;
    assert!(serde_json::from_str::<Expr>(value).is_err());
}
