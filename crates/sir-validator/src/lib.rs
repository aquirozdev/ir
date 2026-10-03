//! Static checks only; successful validation does not imply executable behavior.
use serde::Serialize;
use sir_ir::{Application, FieldType, VERSION};
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

pub fn validate(app: &Application) -> Vec<Diagnostic> {
    let mut errors = Vec::new();
    let mut add = |code, path: String, message: String| {
        errors.push(Diagnostic {
            code,
            path,
            message,
        });
    };
    if app.sir_version != VERSION {
        add(
            "UNSUPPORTED_VERSION",
            "/sir_version".into(),
            format!("expected {VERSION}"),
        );
    }
    if !identifier(&app.name) {
        add(
            "INVALID_IDENTIFIER",
            "/name".into(),
            "invalid application name".into(),
        );
    }
    if app.entities.is_empty() {
        add(
            "EMPTY_APPLICATION",
            "/entities".into(),
            "at least one entity is required".into(),
        );
    }
    let mut names = HashSet::new();
    for (i, entity) in app.entities.iter().enumerate() {
        if !identifier(&entity.name) {
            add(
                "INVALID_IDENTIFIER",
                format!("/entities/{i}/name"),
                "invalid entity name".into(),
            );
        }
        if !names.insert(entity.name.as_str()) {
            add(
                "DUPLICATE_ENTITY",
                format!("/entities/{i}/name"),
                entity.name.clone(),
            );
        }
    }
    for (i, entity) in app.entities.iter().enumerate() {
        if entity.fields.is_empty() {
            add(
                "EMPTY_ENTITY",
                format!("/entities/{i}/fields"),
                "at least one field is required".into(),
            );
        }
        let mut fields = HashSet::new();
        for (j, field) in entity.fields.iter().enumerate() {
            let path = format!("/entities/{i}/fields/{j}");
            if !identifier(&field.name) {
                add(
                    "INVALID_IDENTIFIER",
                    format!("{path}/name"),
                    "invalid field name".into(),
                );
            }
            if !fields.insert(field.name.as_str()) {
                add(
                    "DUPLICATE_FIELD",
                    format!("{path}/name"),
                    field.name.clone(),
                );
            }
            if let FieldType::Ref { entity: target } = &field.field_type {
                if !names.contains(target.as_str()) {
                    add(
                        "UNKNOWN_ENTITY",
                        format!("{path}/field_type/entity"),
                        target.clone(),
                    );
                }
            }
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> Application {
        serde_json::from_str(include_str!("../../../examples/expenses.entities.json")).unwrap()
    }
    #[test]
    fn valid_forward_and_self_references() {
        assert!(validate(&sample()).is_empty());
    }
    #[test]
    fn unknown_reference_is_rejected() {
        let mut app = sample();
        app.entities[0].fields[0].field_type = FieldType::Ref {
            entity: "Missing".into(),
        };
        assert!(validate(&app).iter().any(|e| e.code == "UNKNOWN_ENTITY"));
    }
    #[test]
    fn duplicate_names_are_rejected() {
        let mut app = sample();
        app.entities[1].name = app.entities[0].name.clone();
        app.entities[0].fields[1].name = app.entities[0].fields[0].name.clone();
        let codes: Vec<_> = validate(&app).iter().map(|e| e.code).collect();
        assert!(codes.contains(&"DUPLICATE_ENTITY"));
        assert!(codes.contains(&"DUPLICATE_FIELD"));
    }
    #[test]
    fn invalid_version_and_empty_application_are_rejected() {
        let mut app = sample();
        app.sir_version = "future".into();
        app.entities.clear();
        assert_eq!(validate(&app).len(), 2);
    }
    #[test]
    fn invalid_identifier_and_empty_entity_are_rejected() {
        let mut app = sample();
        app.name = "bad name".into();
        app.entities[0].fields.clear();
        let codes: Vec<_> = validate(&app).iter().map(|e| e.code).collect();
        assert!(codes.contains(&"INVALID_IDENTIFIER"));
        assert!(codes.contains(&"EMPTY_ENTITY"));
    }
    #[test]
    fn unknown_json_fields_are_rejected() {
        let mut value = serde_json::to_value(sample()).unwrap();
        value["commands"] = serde_json::json!([]);
        assert!(serde_json::from_value::<Application>(value).is_err());
    }
}
