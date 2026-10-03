//! Typed, versioned declarations for a bounded business-command interpreter.
use serde::{Deserialize, Serialize};

pub const VERSION: &str = "0.2-draft";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub sir_version: String,
    pub name: String,
    pub entities: Vec<Entity>,
    pub commands: Vec<Command>,
    pub invariants: Vec<Invariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FieldType {
    String,
    Int,
    Bool,
    Ref { entity: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub name: String,
    pub actor: String,
    pub inputs: Vec<Field>,
    /// Required boolean expression. No implicit permit policy exists.
    pub policy: Expr,
    pub requires: Vec<Expr>,
    pub effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invariant {
    pub name: String,
    pub entity: String,
    /// Evaluated for every record of `entity`, bound to `record`.
    pub expression: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expr {
    Path { segments: Vec<String> },
    Literal { value: Literal },
    Eq { left: Box<Expr>, right: Box<Expr> },
    Ne { left: Box<Expr>, right: Box<Expr> },
    Gt { left: Box<Expr>, right: Box<Expr> },
    Gte { left: Box<Expr>, right: Box<Expr> },
    Lt { left: Box<Expr>, right: Box<Expr> },
    Lte { left: Box<Expr>, right: Box<Expr> },
    And { args: Vec<Expr> },
    Or { args: Vec<Expr> },
    Not { arg: Box<Expr> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Literal {
    String(String),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Effect {
    Set { target: Vec<String>, value: Expr },
}
