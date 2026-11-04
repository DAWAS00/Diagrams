# ADR-002 — Native project storage

Status: proposed. Date: 2026-11-04.

## Context

A project has stable semantic IDs, independent views, members, relationships, styles and versioned schema. The basic app must save/reopen reliably and preserve prior committed data after a failed write.

## Alternatives

| Option | Benefit | Cost |
|---|---|---|
| JSON document | Simple debugging and portability | Whole-file replacement/recovery and referential validation remain application responsibilities |
| ZIP + JSON/resources | Portable packaged assets | More archive/update complexity before we need attachments |
| SQLite | Transactions, references, versioned relational structure, local single-file workflow | Native dependency; schema/migration, journaling and publication must be designed/tested |

## Proposed decision

Use SQLite through rusqlite's bundled option. Keep our domain independent of SQL and store an explicit schema version. Use an independently defined `.dgram` format. [rusqlite](https://github.com/rusqlite/rusqlite) documents the Rust binding and bundled SQLite build.

The VP project survey supports separating semantic and visual records. It does not require us to copy VP's schema, encoded definitions or compatibility behavior.

## Consequences

We gain an all-or-nothing persistence mechanism, but must still handle failed initial publication, Save As replacement, disk-full conditions, migration backups and asynchronous revision acknowledgment. Use one worker-owned connection; make journal/checkpoint policy a tested M1 decision. Do not equate a transaction with complete application recovery.

Keep portable typed snapshots at the adapter boundary so JSON interchange/testing remains easy and the storage format does not determine the GUI.

## Revisit trigger

If Git-friendly text review or cross-tool document editing becomes an explicit first-release requirement, reassess JSON as the primary or an additional canonical interchange format. Large embedded assets or collaboration require separate storage proposals.
