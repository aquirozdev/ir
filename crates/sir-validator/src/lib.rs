//! Static symbol, expression, policy and effect validation.
use serde::Serialize;
use sir_ir::{Application, Effect, Expr, Field, FieldType, Literal, VERSION};
use std::collections::{BTreeMap, HashSet};

pub const MAX_DEPTH: usize = 16;
pub const MAX_PATH: usize = 8;
pub const MAX_ENTITIES: usize = 64;
pub const MAX_FIELDS: usize = 64;
pub const MAX_COMMANDS: usize = 64;
pub const MAX_EFFECTS: usize = 64;
pub const MAX_INVARIANTS: usize = 64;
pub const MAX_TERMS: usize = 64;

#[derive(Debug, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
    pub message: String,
}

pub fn identifier(value: &str) -> bool {
    let mut chars = value.chars();
    value.len() <= 64
        && matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

struct Checker<'a> {
    app: &'a Application,
    errors: Vec<Diagnostic>,
}

impl Checker<'_> {
    fn error(&mut self, code: &'static str, path: &str, message: impl Into<String>) {
        self.errors.push(Diagnostic {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
    fn names(&mut self, names: &[&str], path: &str, reserved: bool) {
        let mut seen = HashSet::new();
        for (i, name) in names.iter().enumerate() {
            let location = format!("{path}/{i}/name");
            if !identifier(name) || (reserved && matches!(*name, "actor" | "record")) {
                self.error("INVALID_IDENTIFIER", &location, *name);
            }
            if !seen.insert(name) {
                self.error("DUPLICATE_NAME", &location, *name);
            }
        }
    }
    fn reference(&mut self, entity: &str, path: &str) {
        if !self.app.entities.iter().any(|e| e.name == entity) {
            self.error("UNKNOWN_ENTITY", path, entity);
        }
    }
    fn fields(&mut self, fields: &[Field], path: &str, inputs: bool) {
        if fields.len() > MAX_FIELDS {
            self.error("LIMIT_EXCEEDED", path, "too many fields or inputs");
        }
        self.names(
            &fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
            path,
            inputs,
        );
        for (i, field) in fields.iter().enumerate() {
            if let FieldType::Ref { entity } = &field.field_type {
                self.reference(entity, &format!("{path}/{i}/field_type/entity"));
            }
        }
    }
    fn path_type(
        &mut self,
        segments: &[String],
        env: &BTreeMap<String, FieldType>,
        path: &str,
    ) -> Option<FieldType> {
        if segments.is_empty() || segments.len() > MAX_PATH {
            self.error("INVALID_PATH", path, "path length must be 1..8");
            return None;
        }
        let Some(mut ty) = env.get(&segments[0]).cloned() else {
            self.error("UNKNOWN_BINDING", path, &segments[0]);
            return None;
        };
        for name in &segments[1..] {
            let FieldType::Ref { entity } = ty else {
                self.error("INVALID_PATH", path, "cannot traverse a scalar");
                return None;
            };
            let field = self
                .app
                .entities
                .iter()
                .find(|e| e.name == entity)
                .and_then(|e| e.fields.iter().find(|f| f.name == *name));
            let Some(field) = field else {
                self.error("UNKNOWN_FIELD", path, format!("{entity}.{name}"));
                return None;
            };
            ty = field.field_type.clone();
        }
        Some(ty)
    }
    fn expect(&mut self, ty: Option<FieldType>, expected: &FieldType, path: &str) {
        if let Some(ty) = ty {
            if &ty != expected {
                self.error(
                    "TYPE_MISMATCH",
                    path,
                    format!("expected {expected:?}, got {ty:?}"),
                );
            }
        }
    }
    fn expression(
        &mut self,
        expr: &Expr,
        env: &BTreeMap<String, FieldType>,
        path: &str,
        depth: usize,
    ) -> Option<FieldType> {
        if depth > MAX_DEPTH {
            self.error("LIMIT_EXCEEDED", path, "expression depth exceeds 16");
            return None;
        }
        match expr {
            Expr::Path { segments } => self.path_type(segments, env, path),
            Expr::Literal { value } => Some(match value {
                Literal::String(value) => {
                    if value.len() > 4096 {
                        self.error("LIMIT_EXCEEDED", path, "string literal exceeds 4096 bytes");
                    }
                    FieldType::String
                }
                Literal::Int(_) => FieldType::Int,
                Literal::Bool(_) => FieldType::Bool,
            }),
            Expr::Eq { left, right } | Expr::Ne { left, right } => {
                let left = self.expression(left, env, &format!("{path}/left"), depth + 1);
                let right = self.expression(right, env, &format!("{path}/right"), depth + 1);
                if left.is_some() && right.is_some() && left != right {
                    self.error("TYPE_MISMATCH", path, "equality requires identical types");
                }
                Some(FieldType::Bool)
            }
            Expr::Gt { left, right }
            | Expr::Gte { left, right }
            | Expr::Lt { left, right }
            | Expr::Lte { left, right } => {
                let left = self.expression(left, env, &format!("{path}/left"), depth + 1);
                let right = self.expression(right, env, &format!("{path}/right"), depth + 1);
                self.expect(left, &FieldType::Int, path);
                self.expect(right, &FieldType::Int, path);
                Some(FieldType::Bool)
            }
            Expr::And { args } | Expr::Or { args } => {
                if args.is_empty() || args.len() > MAX_TERMS {
                    self.error(
                        "LIMIT_EXCEEDED",
                        path,
                        "boolean arguments must number 1..64",
                    );
                }
                for (i, arg) in args.iter().enumerate() {
                    let ty = self.expression(arg, env, &format!("{path}/args/{i}"), depth + 1);
                    self.expect(ty, &FieldType::Bool, path);
                }
                Some(FieldType::Bool)
            }
            Expr::Not { arg } => {
                let ty = self.expression(arg, env, &format!("{path}/arg"), depth + 1);
                self.expect(ty, &FieldType::Bool, path);
                Some(FieldType::Bool)
            }
        }
    }
    fn boolean(&mut self, expr: &Expr, env: &BTreeMap<String, FieldType>, path: &str) {
        let ty = self.expression(expr, env, path, 0);
        self.expect(ty, &FieldType::Bool, path);
    }
}

pub fn validate(app: &Application) -> Vec<Diagnostic> {
    let mut c = Checker {
        app,
        errors: Vec::new(),
    };
    if app.sir_version != VERSION {
        c.error(
            "UNSUPPORTED_VERSION",
            "/sir_version",
            format!("expected {VERSION}"),
        );
    }
    if !identifier(&app.name) {
        c.error("INVALID_IDENTIFIER", "/name", "invalid application name");
    }
    if app.entities.is_empty() || app.entities.len() > MAX_ENTITIES {
        c.error("LIMIT_EXCEEDED", "/entities", "entities must number 1..64");
    }
    c.names(
        &app.entities
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        "/entities",
        false,
    );
    for (i, entity) in app.entities.iter().enumerate() {
        let path = format!("/entities/{i}/fields");
        if entity.fields.is_empty() {
            c.error("EMPTY_ENTITY", &path, "entity must have fields");
        }
        c.fields(&entity.fields, &path, false);
    }
    if app.commands.len() > MAX_COMMANDS || app.invariants.len() > MAX_INVARIANTS {
        c.error("LIMIT_EXCEEDED", "/", "too many commands or invariants");
    }
    c.names(
        &app.commands
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        "/commands",
        false,
    );
    c.names(
        &app.invariants
            .iter()
            .map(|v| v.name.as_str())
            .collect::<Vec<_>>(),
        "/invariants",
        false,
    );
    for (i, command) in app.commands.iter().enumerate() {
        let path = format!("/commands/{i}");
        c.reference(&command.actor, &format!("{path}/actor"));
        c.fields(&command.inputs, &format!("{path}/inputs"), true);
        let mut env: BTreeMap<_, _> = command
            .inputs
            .iter()
            .map(|f| (f.name.clone(), f.field_type.clone()))
            .collect();
        env.insert(
            "actor".into(),
            FieldType::Ref {
                entity: command.actor.clone(),
            },
        );
        c.boolean(&command.policy, &env, &format!("{path}/policy"));
        if command.requires.len() > MAX_TERMS || command.effects.len() > MAX_EFFECTS {
            c.error("LIMIT_EXCEEDED", &path, "too many preconditions or effects");
        }
        if command.effects.is_empty() {
            c.error(
                "EMPTY_COMMAND",
                &format!("{path}/effects"),
                "command must have effects",
            );
        }
        for (j, expr) in command.requires.iter().enumerate() {
            c.boolean(expr, &env, &format!("{path}/requires/{j}"));
        }
        for (j, effect) in command.effects.iter().enumerate() {
            let location = format!("{path}/effects/{j}");
            let Effect::Set { target, value } = effect;
            if target.len() < 2 || target.first().is_some_and(|v| v == "actor") {
                c.error(
                    "INVALID_EFFECT",
                    &location,
                    "set must target a field rooted in an input reference",
                );
            }
            let target_type = c.path_type(target, &env, &format!("{location}/target"));
            let value_type = c.expression(value, &env, &format!("{location}/value"), 0);
            if let Some(target_type) = target_type {
                c.expect(value_type, &target_type, &location);
            }
        }
    }
    for (i, invariant) in app.invariants.iter().enumerate() {
        let path = format!("/invariants/{i}");
        c.reference(&invariant.entity, &format!("{path}/entity"));
        let env = BTreeMap::from([(
            "record".into(),
            FieldType::Ref {
                entity: invariant.entity.clone(),
            },
        )]);
        c.boolean(&invariant.expression, &env, &format!("{path}/expression"));
    }
    c.errors
}
