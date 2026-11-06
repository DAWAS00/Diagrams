//! Schema v1 of a project file.
//!
//! A project is one SQLite database. `application_id` marks it as ours and
//! `user_version` carries the schema version, so both can be read before any
//! table is touched. Foreign keys and checks are part of the schema, but
//! loading never trusts them: every row is validated again in Rust through
//! the core's own rules.

/// The newest schema this build writes and reads.
pub const SCHEMA_VERSION: u32 = 1;

/// Marks a SQLite file as a Diagram Studio project ("DGR1").
pub const APPLICATION_ID: i32 = 0x4447_5231;

/// Creates every table of schema v1.
pub const CREATE_V1: &str = "
CREATE TABLE meta (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
) WITHOUT ROWID;
CREATE TABLE elements (
    id   INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0)
);
CREATE TABLE members (
    id           INTEGER NOT NULL PRIMARY KEY,
    element_id   INTEGER NOT NULL REFERENCES elements(id),
    position     INTEGER NOT NULL,
    kind         TEXT NOT NULL CHECK (kind IN ('attribute', 'operation')),
    name         TEXT NOT NULL,
    visibility   TEXT NOT NULL,
    type_text    TEXT,
    multiplicity TEXT,
    return_type  TEXT,
    UNIQUE (element_id, position)
);
CREATE TABLE parameters (
    member_id INTEGER NOT NULL REFERENCES members(id),
    position  INTEGER NOT NULL,
    name      TEXT NOT NULL,
    type_text TEXT NOT NULL,
    PRIMARY KEY (member_id, position)
) WITHOUT ROWID;
CREATE TABLE relations (
    id                  INTEGER NOT NULL PRIMARY KEY,
    kind                TEXT NOT NULL CHECK (kind IN ('association', 'generalization')),
    source              INTEGER NOT NULL REFERENCES elements(id),
    target              INTEGER NOT NULL REFERENCES elements(id),
    source_role         TEXT,
    source_multiplicity TEXT,
    target_role         TEXT,
    target_multiplicity TEXT
);
CREATE TABLE diagrams (
    id       INTEGER NOT NULL PRIMARY KEY,
    position INTEGER NOT NULL UNIQUE,
    name     TEXT NOT NULL,
    notation TEXT NOT NULL
);
CREATE TABLE node_views (
    id         INTEGER NOT NULL PRIMARY KEY,
    diagram_id INTEGER NOT NULL REFERENCES diagrams(id),
    position   INTEGER NOT NULL,
    element_id INTEGER NOT NULL REFERENCES elements(id),
    x REAL NOT NULL, y REAL NOT NULL, width REAL NOT NULL, height REAL NOT NULL,
    UNIQUE (diagram_id, position)
);
CREATE TABLE edge_views (
    id          INTEGER NOT NULL PRIMARY KEY,
    diagram_id  INTEGER NOT NULL REFERENCES diagrams(id),
    position    INTEGER NOT NULL,
    relation_id INTEGER NOT NULL REFERENCES relations(id),
    source_view INTEGER NOT NULL REFERENCES node_views(id),
    target_view INTEGER NOT NULL REFERENCES node_views(id),
    UNIQUE (diagram_id, position)
);
";
