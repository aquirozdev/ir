//! Transactional interpreter. Identity is supplied by a trusted host, not inferred from input JSON.
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sir_ir::{Application, Effect, Expr, FieldType, Literal};
use std::{collections::BTreeMap, fmt, path::Path, time::Duration};

pub const MAX_RECORDS: usize = 10_000;
pub const MAX_STRING_BYTES: usize = 4096;
pub const MAX_STATE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_PROGRAM_BYTES: usize = 1024 * 1024;
pub const MAX_STEPS: usize = 1_000_000;

#[derive(Debug, Serialize)]
pub struct Error {
    pub code: &'static str,
    pub message: String,
}
impl Error {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Error {}
impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::new("STORAGE_ERROR", error.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::new("JSON_ERROR", error.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub entity: String,
    pub id: String,
    pub fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    pub entity: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct Execution {
    pub changed: Vec<Record>,
}

type State = BTreeMap<(String, String), Record>;
#[derive(Debug, Clone, PartialEq, Eq)]
enum Data {
    String(String),
    Int(i64),
    Bool(bool),
    Ref(Principal),
}
impl Data {
    fn wire(&self) -> Value {
        match self {
            Self::String(v) => Value::String(v.clone()),
            Self::Int(v) => Value::from(*v),
            Self::Bool(v) => Value::Bool(*v),
            Self::Ref(v) => Value::String(v.id.clone()),
        }
    }
    fn boolean(&self) -> Result<bool> {
        match self {
            Self::Bool(v) => Ok(*v),
            _ => Err(Error::new("TYPE_MISMATCH", "expected boolean")),
        }
    }
    fn integer(&self) -> Result<i64> {
        match self {
            Self::Int(v) => Ok(*v),
            _ => Err(Error::new("TYPE_MISMATCH", "expected integer")),
        }
    }
    fn reference(&self) -> Result<&Principal> {
        match self {
            Self::Ref(v) => Ok(v),
            _ => Err(Error::new("TYPE_MISMATCH", "expected record reference")),
        }
    }
}

fn record<'a>(state: &'a State, reference: &Principal) -> Result<&'a Record> {
    state
        .get(&(reference.entity.clone(), reference.id.clone()))
        .ok_or_else(|| Error::new("NOT_FOUND", "record does not exist"))
}
fn decode(value: &Value, ty: &FieldType) -> Result<Data> {
    let data = match ty {
        FieldType::String => value
            .as_str()
            .filter(|s| s.len() <= MAX_STRING_BYTES)
            .map(|v| Data::String(v.into())),
        FieldType::Int => value.as_i64().map(Data::Int),
        FieldType::Bool => value.as_bool().map(Data::Bool),
        FieldType::Ref { entity } => value
            .as_str()
            .filter(|id| sir_validator::identifier(id))
            .map(|id| {
                Data::Ref(Principal {
                    entity: entity.clone(),
                    id: id.into(),
                })
            }),
    };
    data.ok_or_else(|| {
        Error::new(
            "TYPE_MISMATCH",
            "value does not match declared type or size limit",
        )
    })
}
fn field_type<'a>(app: &'a Application, entity: &str, name: &str) -> Result<&'a FieldType> {
    app.entities
        .iter()
        .find(|e| e.name == entity)
        .and_then(|e| e.fields.iter().find(|f| f.name == name))
        .map(|f| &f.field_type)
        .ok_or_else(|| Error::new("UNKNOWN_FIELD", "field is not declared"))
}
fn path_value(
    app: &Application,
    state: &State,
    env: &BTreeMap<String, Data>,
    segments: &[String],
) -> Result<Data> {
    let root = segments
        .first()
        .ok_or_else(|| Error::new("INVALID_PATH", "empty path"))?;
    let mut value = env
        .get(root)
        .cloned()
        .ok_or_else(|| Error::new("UNKNOWN_BINDING", "path root is unbound"))?;
    for name in &segments[1..] {
        let reference = value.reference()?;
        let row = record(state, reference)?;
        let raw = row
            .fields
            .get(name)
            .ok_or_else(|| Error::new("UNKNOWN_FIELD", "missing record field"))?;
        value = decode(raw, field_type(app, &reference.entity, name)?)?;
    }
    Ok(value)
}
fn eval(
    app: &Application,
    state: &State,
    env: &BTreeMap<String, Data>,
    expr: &Expr,
    budget: &mut usize,
) -> Result<Data> {
    if *budget == 0 {
        return Err(Error::new(
            "LIMIT_EXCEEDED",
            "expression step budget exhausted",
        ));
    }
    *budget -= 1;
    match expr {
        Expr::Path { segments } => path_value(app, state, env, segments),
        Expr::Literal { value } => Ok(match value {
            Literal::String(v) => Data::String(v.clone()),
            Literal::Int(v) => Data::Int(*v),
            Literal::Bool(v) => Data::Bool(*v),
        }),
        Expr::Eq { left, right } | Expr::Ne { left, right } => {
            let equal =
                eval(app, state, env, left, budget)? == eval(app, state, env, right, budget)?;
            Ok(Data::Bool(if matches!(expr, Expr::Eq { .. }) {
                equal
            } else {
                !equal
            }))
        }
        Expr::Gt { left, right }
        | Expr::Gte { left, right }
        | Expr::Lt { left, right }
        | Expr::Lte { left, right } => {
            let a = eval(app, state, env, left, budget)?.integer()?;
            let b = eval(app, state, env, right, budget)?.integer()?;
            Ok(Data::Bool(match expr {
                Expr::Gt { .. } => a > b,
                Expr::Gte { .. } => a >= b,
                Expr::Lt { .. } => a < b,
                Expr::Lte { .. } => a <= b,
                _ => unreachable!(),
            }))
        }
        Expr::And { args } => {
            for arg in args {
                if !eval(app, state, env, arg, budget)?.boolean()? {
                    return Ok(Data::Bool(false));
                }
            }
            Ok(Data::Bool(true))
        }
        Expr::Or { args } => {
            for arg in args {
                if eval(app, state, env, arg, budget)?.boolean()? {
                    return Ok(Data::Bool(true));
                }
            }
            Ok(Data::Bool(false))
        }
        Expr::Not { arg } => Ok(Data::Bool(!eval(app, state, env, arg, budget)?.boolean()?)),
    }
}
fn validate_state(app: &Application, state: &State) -> Result<()> {
    if state.len() > MAX_RECORDS {
        return Err(Error::new("LIMIT_EXCEEDED", "too many records"));
    }
    let mut bytes = 0;
    for row in state.values() {
        bytes += serde_json::to_vec(row)?.len();
        if bytes > MAX_STATE_BYTES {
            return Err(Error::new("LIMIT_EXCEEDED", "state exceeds byte limit"));
        }
        if !sir_validator::identifier(&row.id) {
            return Err(Error::new(
                "INVALID_ID",
                "record id must be an identifier of at most 64 bytes",
            ));
        }
        let entity = app
            .entities
            .iter()
            .find(|e| e.name == row.entity)
            .ok_or_else(|| Error::new("UNKNOWN_ENTITY", "record entity is undeclared"))?;
        if row.fields.len() != entity.fields.len() {
            return Err(Error::new(
                "INVALID_RECORD",
                "record has missing or additional fields",
            ));
        }
        for field in &entity.fields {
            let raw = row
                .fields
                .get(&field.name)
                .ok_or_else(|| Error::new("INVALID_RECORD", "missing field"))?;
            if let Data::Ref(reference) = decode(raw, &field.field_type)? {
                record(state, &reference)?;
            }
        }
    }
    Ok(())
}
fn invariants(app: &Application, state: &State, budget: &mut usize) -> Result<()> {
    for invariant in &app.invariants {
        for row in state.values().filter(|row| row.entity == invariant.entity) {
            let env = BTreeMap::from([(
                "record".into(),
                Data::Ref(Principal {
                    entity: row.entity.clone(),
                    id: row.id.clone(),
                }),
            )]);
            if !eval(app, state, &env, &invariant.expression, budget)?.boolean()? {
                return Err(Error::new(
                    "INVARIANT_FAILED",
                    format!("invariant {} failed", invariant.name),
                ));
            }
        }
    }
    Ok(())
}
fn load(conn: &Connection) -> Result<State> {
    let mut statement =
        conn.prepare("SELECT entity, id, data FROM sir_records ORDER BY entity, id LIMIT ?1")?;
    let rows = statement.query_map([MAX_RECORDS as i64 + 1], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut state = State::new();
    let mut bytes = 0;
    for row in rows {
        let (entity, id, data) = row?;
        bytes += entity.len() + id.len() + data.len();
        if bytes > MAX_STATE_BYTES || state.len() == MAX_RECORDS {
            return Err(Error::new(
                "LIMIT_EXCEEDED",
                "stored state exceeds configured limit",
            ));
        }
        let fields = serde_json::from_str(&data)?;
        state.insert((entity.clone(), id.clone()), Record { entity, id, fields });
    }
    Ok(state)
}

pub struct Runtime {
    app: Application,
    conn: Connection,
}
impl Runtime {
    pub fn open(app: Application, path: impl AsRef<Path>) -> Result<Self> {
        Self::install(app, Connection::open(path)?)
    }
    pub fn memory(app: Application) -> Result<Self> {
        Self::install(app, Connection::open_in_memory()?)
    }
    fn install(app: Application, mut conn: Connection) -> Result<Self> {
        let program = serde_json::to_string(&app)?;
        if program.len() > MAX_PROGRAM_BYTES {
            return Err(Error::new("LIMIT_EXCEEDED", "program exceeds byte limit"));
        }
        let diagnostics = sir_validator::validate(&app);
        if !diagnostics.is_empty() {
            return Err(Error::new(
                "INVALID_PROGRAM",
                serde_json::to_string(&diagnostics)?,
            ));
        }
        conn.busy_timeout(Duration::from_secs(5))?;
        // Fixed schema and parameterized SQL. Model output cannot supply SQL.
        conn.execute_batch("CREATE TABLE IF NOT EXISTS sir_metadata (id INTEGER PRIMARY KEY CHECK (id = 1), program TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS sir_records (entity TEXT NOT NULL, id TEXT NOT NULL, data TEXT NOT NULL, PRIMARY KEY(entity, id));")?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let installed: Option<String> = tx
            .query_row("SELECT program FROM sir_metadata WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()?;
        match installed {
            Some(existing) if existing != program => {
                return Err(Error::new(
                    "PROGRAM_MISMATCH",
                    "database belongs to a different program; migrations are not supported",
                ))
            }
            None => {
                tx.execute(
                    "INSERT INTO sir_metadata (id, program) VALUES (1, ?1)",
                    [&program],
                )?;
            }
            _ => {}
        }
        tx.commit()?;
        Ok(Self { app, conn })
    }
    /// Administrative initialization, separate from untrusted command execution.
    pub fn seed(&mut self, records: Vec<Record>) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let count: i64 = tx.query_row("SELECT COUNT(*) FROM sir_records", [], |row| row.get(0))?;
        if count != 0 {
            return Err(Error::new(
                "ALREADY_INITIALIZED",
                "seeding requires an empty database",
            ));
        }
        if records.len() > MAX_RECORDS {
            return Err(Error::new("LIMIT_EXCEEDED", "too many seed records"));
        }
        let mut state = State::new();
        for row in records {
            if state
                .insert((row.entity.clone(), row.id.clone()), row)
                .is_some()
            {
                return Err(Error::new("DUPLICATE_RECORD", "duplicate entity/id"));
            }
        }
        validate_state(&self.app, &state)?;
        let mut budget = MAX_STEPS;
        invariants(&self.app, &state, &mut budget)?;
        for row in state.values() {
            tx.execute(
                "INSERT INTO sir_records (entity, id, data) VALUES (?1, ?2, ?3)",
                params![row.entity, row.id, serde_json::to_string(&row.fields)?],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    /// Administrative snapshot. A remote adapter must authorize reads separately.
    pub fn snapshot(&self) -> Result<Vec<Record>> {
        let state = load(&self.conn)?;
        validate_state(&self.app, &state)?;
        Ok(state.into_values().collect())
    }
    pub fn execute(
        &mut self,
        name: &str,
        actor: Principal,
        inputs: BTreeMap<String, Value>,
    ) -> Result<Execution> {
        let command = self
            .app
            .commands
            .iter()
            .find(|c| c.name == name)
            .ok_or_else(|| Error::new("UNKNOWN_COMMAND", "command is undeclared"))?;
        if actor.entity != command.actor || !sir_validator::identifier(&actor.id) {
            return Err(Error::new(
                "UNAUTHORIZED",
                "principal type or id is invalid",
            ));
        }
        if inputs.len() != command.inputs.len() {
            return Err(Error::new(
                "INVALID_INPUT",
                "missing or additional input keys",
            ));
        }
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut state = load(&tx)?;
        validate_state(&self.app, &state)?;
        record(&state, &actor)?;
        let mut env = BTreeMap::from([("actor".into(), Data::Ref(actor))]);
        for input in &command.inputs {
            let raw = inputs
                .get(&input.name)
                .ok_or_else(|| Error::new("INVALID_INPUT", "missing input key"))?;
            let data = decode(raw, &input.field_type)?;
            if let Data::Ref(reference) = &data {
                record(&state, reference)?;
            }
            env.insert(input.name.clone(), data);
        }
        let mut budget = MAX_STEPS;
        if !eval(&self.app, &state, &env, &command.policy, &mut budget)?.boolean()? {
            return Err(Error::new("UNAUTHORIZED", "policy denied the command"));
        }
        for expr in &command.requires {
            if !eval(&self.app, &state, &env, expr, &mut budget)?.boolean()? {
                return Err(Error::new(
                    "PRECONDITION_FAILED",
                    "command precondition failed",
                ));
            }
        }
        let before = state.clone();
        for effect in &command.effects {
            let Effect::Set { target, value } = effect;
            let reference = path_value(&self.app, &state, &env, &target[..target.len() - 1])?
                .reference()?
                .clone();
            let new_value = eval(&self.app, &state, &env, value, &mut budget)?;
            let field = &target[target.len() - 1];
            state
                .get_mut(&(reference.entity, reference.id))
                .ok_or_else(|| Error::new("NOT_FOUND", "effect target does not exist"))?
                .fields
                .insert(field.clone(), new_value.wire());
        }
        validate_state(&self.app, &state)?;
        invariants(&self.app, &state, &mut budget)?;
        let changed: Vec<Record> = state
            .iter()
            .filter(|(key, row)| before.get(*key) != Some(*row))
            .map(|(_, row)| row.clone())
            .collect();
        // Materialize writes before commit, still inside the transaction.
        for row in &changed {
            tx.execute(
                "UPDATE sir_records SET data = ?1 WHERE entity = ?2 AND id = ?3",
                params![serde_json::to_string(&row.fields)?, row.entity, row.id],
            )?;
        }
        tx.commit()?;
        Ok(Execution { changed })
    }
}
