//! SQLite bootstrap and migration engine.
//!
//! Scope (blueprint Phase 9, WP 9.8): SQLite stores cache/metrics/usage/run rows;
//! the graph lives in memory plus versioned JSON snapshots. This module owns the
//! connection, pragmas and the ordered migration list; later phases add
//! migrations rather than editing earlier ones.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension};
use thiserror::Error;

use hiverouter_contract::event::ToolOutcome;
use hiverouter_contract::{Refusal, RunId, TaskId, Timestamp};

/// One ordered schema migration with an exercised inverse.
#[derive(Clone, Copy, Debug)]
pub struct Migration {
    /// Monotonic schema version starting at 1.
    pub version: i64,
    /// Short migration name recorded in `schema_migrations`.
    pub name: &'static str,
    /// Forward SQL.
    pub up: &'static str,
    /// Inverse SQL (exercised by the migration-roundtrip test).
    pub down: &'static str,
}

/// Ordered migrations. Append only; never edit a migration that has shipped.
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    name: "bootstrap",
    up: r#"
        CREATE TABLE IF NOT EXISTS app_meta (
            key   TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        INSERT INTO app_meta(key, value) VALUES ('schema_version', 'hiverouter.v1')
            ON CONFLICT(key) DO UPDATE SET value = excluded.value;
        INSERT INTO app_meta(key, value) VALUES ('bootstrapped_at', strftime('%Y-%m-%dT%H:%M:%SZ','now'))
            ON CONFLICT(key) DO NOTHING;
    "#,
    down: r#"
        DROP TABLE IF EXISTS app_meta;
    "#,
},
Migration {
    version: 2,
    name: "runs_tasks_chat",
    up: r#"
        CREATE TABLE IF NOT EXISTS runs (
            id          TEXT PRIMARY KEY NOT NULL,
            project_id  TEXT NOT NULL,
            prompt      TEXT NOT NULL,
            status      TEXT NOT NULL,
            error_code  TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status);
        CREATE TABLE IF NOT EXISTS tasks (
            id             TEXT PRIMARY KEY NOT NULL,
            run_id         TEXT NOT NULL,
            parent_task_id TEXT,
            role           TEXT NOT NULL,
            depth          INTEGER NOT NULL,
            spawn_seq      INTEGER NOT NULL,
            state          TEXT NOT NULL,
            rev            INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_run ON tasks(run_id);
        CREATE TABLE IF NOT EXISTS chat_messages (
            id         TEXT PRIMARY KEY NOT NULL,
            run_id     TEXT NOT NULL,
            role       TEXT NOT NULL,
            text       TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_messages_run ON chat_messages(run_id);
        CREATE TABLE IF NOT EXISTS command_receipts (
            correlation_id TEXT PRIMARY KEY NOT NULL,
            run_id         TEXT NOT NULL,
            created_at     TEXT NOT NULL
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS command_receipts;
        DROP TABLE IF EXISTS chat_messages;
        DROP TABLE IF EXISTS tasks;
        DROP TABLE IF EXISTS runs;
    "#,
},
Migration {
    version: 3,
    name: "snapshot_index",
    up: r#"
        CREATE TABLE IF NOT EXISTS snapshots (
            run_id         TEXT NOT NULL,
            revision       INTEGER NOT NULL,
            path           TEXT NOT NULL,
            created_at     TEXT NOT NULL,
            schema_version TEXT NOT NULL,
            PRIMARY KEY (run_id, revision)
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS snapshots;
    "#,
},
Migration {
    version: 4,
    name: "tool_turns",
    up: r#"
        ALTER TABLE runs ADD COLUMN tool_turns INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE runs ADD COLUMN tool_calls INTEGER NOT NULL DEFAULT 0;
        CREATE TABLE IF NOT EXISTS tool_turns (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            task_id TEXT NOT NULL,
            tool_call_id TEXT NOT NULL,
            tool TEXT NOT NULL,
            args_sha256 TEXT NOT NULL,
            outcome TEXT NOT NULL,
            code TEXT,
            bytes INTEGER,
            sha256 TEXT,
            turn INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(run_id, tool_call_id)
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS tool_turns;
        ALTER TABLE runs DROP COLUMN tool_calls;
        ALTER TABLE runs DROP COLUMN tool_turns;
    "#,
},
Migration {
    version: 5,
    name: "task_results",
    up: r#"
        CREATE TABLE IF NOT EXISTS task_results (
            task_id    TEXT PRIMARY KEY NOT NULL,
            run_id     TEXT NOT NULL,
            text       TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_task_results_run ON task_results(run_id);
    "#,
    down: r#"
        DROP TABLE IF EXISTS task_results;
    "#,
},
Migration {
    version: 6,
    name: "agent_messages",
    up: r#"
        CREATE TABLE IF NOT EXISTS agent_messages (
            id             TEXT PRIMARY KEY NOT NULL,
            run_id         TEXT NOT NULL,
            from_task_id   TEXT NOT NULL,
            to_task_id     TEXT NOT NULL,
            kind           TEXT NOT NULL,
            text           TEXT NOT NULL,
            created_at     TEXT NOT NULL,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_agent_messages_run ON agent_messages(run_id);
    "#,
    down: r#"
        DROP TABLE IF EXISTS agent_messages;
    "#,
},
Migration {
    version: 7,
    name: "agent_memory",
    up: r#"
        CREATE TABLE IF NOT EXISTS agent_memory (
            agent_key   TEXT NOT NULL,
            key         TEXT NOT NULL,
            value       TEXT NOT NULL,
            run_id      TEXT NOT NULL,
            task_id     TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            PRIMARY KEY (agent_key, key)
        );
        CREATE INDEX IF NOT EXISTS idx_agent_memory_agent ON agent_memory(agent_key);
    "#,
    down: r#"
        DROP TABLE IF EXISTS agent_memory;
    "#,
},
Migration {
    version: 8,
    name: "run_usage",
    up: r#"
        CREATE TABLE IF NOT EXISTS run_usage (
            run_id              TEXT PRIMARY KEY NOT NULL,
            prompt_tokens       INTEGER,
            completion_tokens   INTEGER,
            total_tokens        INTEGER,
            prompt_cache_hit_tokens  INTEGER,
            prompt_cache_miss_tokens INTEGER,
            reasoning_tokens    INTEGER,
            created_at          TEXT NOT NULL,
            FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS run_usage;
    "#,
},
Migration {
    version: 9,
    name: "agent_profiles",
    up: r#"
        CREATE TABLE IF NOT EXISTS agent_profiles (
            id          TEXT PRIMARY KEY NOT NULL,
            profile_json TEXT NOT NULL,
            source      TEXT NOT NULL,
            imported_at TEXT NOT NULL
        );
    "#,
    down: r#"
        DROP TABLE IF EXISTS agent_profiles;
    "#,
},
Migration {
    version: 10,
    name: "projects",
    up: r#"
        CREATE TABLE IF NOT EXISTS projects (
            id          TEXT PRIMARY KEY NOT NULL,
            name        TEXT NOT NULL,
            created_at  TEXT NOT NULL
        );
        INSERT OR IGNORE INTO projects(id, name, created_at)
        SELECT DISTINCT project_id, project_id, MIN(created_at) FROM runs GROUP BY project_id;
    "#,
    down: r#"
        DROP TABLE IF EXISTS projects;
    "#,
},
Migration {
    version: 11,
    name: "agent_memory_scoped",
    up: r#"
        CREATE TABLE agent_memory_scoped (
            run_id      TEXT NOT NULL,
            scope_id    TEXT NOT NULL,
            agent_key   TEXT NOT NULL,
            key         TEXT NOT NULL,
            value       TEXT NOT NULL,
            task_id     TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            PRIMARY KEY (run_id, scope_id, agent_key, key)
        );
        INSERT OR REPLACE INTO agent_memory_scoped(run_id, scope_id, agent_key, key, value, task_id, created_at)
        SELECT run_id, 'legacy', agent_key, key, value, task_id, created_at FROM agent_memory;
        DROP TABLE agent_memory;
        ALTER TABLE agent_memory_scoped RENAME TO agent_memory;
        CREATE INDEX IF NOT EXISTS idx_agent_memory_agent ON agent_memory(agent_key);
        CREATE INDEX IF NOT EXISTS idx_agent_memory_run ON agent_memory(run_id);
        CREATE INDEX IF NOT EXISTS idx_agent_memory_scope ON agent_memory(run_id, scope_id, agent_key);
    "#,
    down: r#"
        DROP TABLE IF EXISTS agent_memory;
    "#,
},
Migration {
    version: 12,
    name: "a2a_answer_state",
    up: r#"
        ALTER TABLE agent_messages ADD COLUMN status TEXT NOT NULL DEFAULT 'open';
        ALTER TABLE agent_messages ADD COLUMN deadline_at TEXT;
        ALTER TABLE agent_messages ADD COLUMN answered_at TEXT;
        ALTER TABLE agent_messages ADD COLUMN answer_text TEXT;
        ALTER TABLE agent_messages ADD COLUMN answer_actor TEXT;
        ALTER TABLE agent_messages ADD COLUMN in_reply_to TEXT;
        CREATE INDEX IF NOT EXISTS idx_agent_messages_status
            ON agent_messages(run_id, status);
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_agent_messages_status;
        ALTER TABLE agent_messages DROP COLUMN in_reply_to;
        ALTER TABLE agent_messages DROP COLUMN answer_actor;
        ALTER TABLE agent_messages DROP COLUMN answer_text;
        ALTER TABLE agent_messages DROP COLUMN answered_at;
        ALTER TABLE agent_messages DROP COLUMN deadline_at;
        ALTER TABLE agent_messages DROP COLUMN status;
    "#,
},
Migration {
    version: 13,
    name: "task_identity",
    up: r#"
        ALTER TABLE tasks ADD COLUMN class TEXT;
        ALTER TABLE tasks ADD COLUMN rank INTEGER;
        ALTER TABLE tasks ADD COLUMN profile_id TEXT;
        UPDATE tasks SET
            class = CASE
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'hyper'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('super','superagent','super-agent','sub-hive','subhive') THEN 'super'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('subagent','sub-agent') THEN 'subagent'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('agent','worker','workeragent','worker-agent','content-writer',
                     'data-analyst','code-reviewer','chatbot') THEN 'agent'
                ELSE NULL END,
            rank = CASE
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 1
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('super','superagent','super-agent','sub-hive','subhive') THEN 2
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('subagent','sub-agent') THEN 4
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('agent','worker','workeragent','worker-agent','content-writer',
                     'data-analyst','code-reviewer','chatbot') THEN 3
                ELSE NULL END,
            profile_id = CASE
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'planner'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('super','superagent','super-agent','sub-hive','subhive') THEN 'superagent'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('subagent','sub-agent') THEN 'subagent'
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('content-writer','data-analyst','code-reviewer','chatbot')
                    THEN replace(replace(lower(trim(role)),'_','-'),' ','-')
                WHEN replace(replace(lower(trim(role)),'_','-'),' ','-') IN
                    ('agent','worker','workeragent','worker-agent') THEN 'agent'
                ELSE NULL END;
        CREATE TRIGGER IF NOT EXISTS tasks_identity_after_insert
        AFTER INSERT ON tasks
        WHEN NEW.class IS NULL
        BEGIN
            UPDATE tasks SET
                class = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'hyper'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN 'super'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('subagent','sub-agent') THEN 'subagent'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('agent','worker','workeragent','worker-agent','content-writer',
                         'data-analyst','code-reviewer','chatbot') THEN 'agent'
                    ELSE NULL END,
                rank = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 1
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN 2
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('subagent','sub-agent') THEN 4
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('agent','worker','workeragent','worker-agent','content-writer',
                         'data-analyst','code-reviewer','chatbot') THEN 3
                    ELSE NULL END,
                profile_id = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'planner'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN 'superagent'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('subagent','sub-agent') THEN 'subagent'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('content-writer','data-analyst','code-reviewer','chatbot')
                        THEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-')
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('agent','worker','workeragent','worker-agent') THEN 'agent'
                    ELSE NULL END
            WHERE id = NEW.id;
        END;
        CREATE INDEX IF NOT EXISTS idx_tasks_identity ON tasks(run_id, class, profile_id);
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_tasks_identity;
        DROP TRIGGER IF EXISTS tasks_identity_after_insert;
        ALTER TABLE tasks DROP COLUMN profile_id;
        ALTER TABLE tasks DROP COLUMN rank;
        ALTER TABLE tasks DROP COLUMN class;
    "#,
},
Migration {
    version: 14,
    name: "memory_banks",
    up: r#"
        CREATE TABLE memory_banks (
            id             TEXT PRIMARY KEY NOT NULL,
            kind           TEXT NOT NULL CHECK (kind IN
                             ('global','project','run','hive','branch','agent_role')),
            scope_id       TEXT NOT NULL,
            run_id         TEXT,
            owner_task_id  TEXT,
            parent_bank_id TEXT,
            display_name   TEXT,
            rev            INTEGER NOT NULL DEFAULT 0 CHECK (rev >= 0),
            created_at     TEXT NOT NULL,
            closed_at      TEXT,
            UNIQUE (kind, scope_id)
        );
        CREATE TABLE memory_entries (
            id             TEXT PRIMARY KEY NOT NULL,
            bank_id        TEXT NOT NULL,
            entry_key      TEXT NOT NULL,
            kind           TEXT NOT NULL CHECK (kind IN
                             ('observation','summary','fact','decision','constraint',
                              'preference','artifact_ref','note')),
            value          TEXT NOT NULL,
            source_kind    TEXT NOT NULL CHECK (source_kind IN
                             ('agent_result','engine_summary','operator','tool_output',
                              'import','migration','a2a_message','parent_snapshot')),
            source_task_id TEXT,
            source_run_id  TEXT,
            trust          TEXT CHECK (trust IN
                             ('quarantined','unverified','verified','authoritative')),
            status         TEXT NOT NULL DEFAULT 'active'
                             CHECK (status IN ('active','superseded','retracted')),
            created_at     TEXT NOT NULL,
            updated_at     TEXT NOT NULL,
            FOREIGN KEY (bank_id) REFERENCES memory_banks(id) ON DELETE CASCADE
        );
        CREATE UNIQUE INDEX idx_memory_entries_active
            ON memory_entries(bank_id, entry_key) WHERE status = 'active';
        CREATE INDEX idx_memory_banks_run ON memory_banks(run_id);
        CREATE INDEX idx_memory_banks_kind ON memory_banks(kind);
        CREATE INDEX idx_memory_entries_bank ON memory_entries(bank_id, created_at);

        INSERT OR IGNORE INTO memory_banks
            (id, kind, scope_id, run_id, owner_task_id, parent_bank_id,
             display_name, rev, created_at, closed_at)
        VALUES ('bank:global:default', 'global', 'default', NULL, NULL, NULL,
                'Global', 0, strftime('%Y-%m-%dT%H:%M:%SZ','now'), NULL);

        INSERT OR IGNORE INTO memory_banks
            (id, kind, scope_id, run_id, owner_task_id, parent_bank_id,
             display_name, rev, created_at, closed_at)
        SELECT 'bank:run:'||m.run_id, 'run', m.run_id, m.run_id, NULL,
               'bank:global:default', 'Run '||substr(m.run_id,1,8), 0,
               MIN(m.created_at), NULL
        FROM agent_memory m
        WHERE EXISTS (SELECT 1 FROM runs r WHERE r.id = m.run_id)
        GROUP BY m.run_id;

        INSERT OR IGNORE INTO memory_banks
            (id, kind, scope_id, run_id, owner_task_id, parent_bank_id,
             display_name, rev, created_at, closed_at)
        SELECT 'bank:agent_role:'||
               replace(replace(lower(trim(m.agent_key)),'-','_'),' ','_'),
               'agent_role',
               replace(replace(lower(trim(m.agent_key)),'-','_'),' ','_'),
               NULL, NULL, NULL, m.agent_key, 0, MIN(m.created_at), NULL
        FROM agent_memory m
        WHERE m.scope_id = 'legacy'
           OR NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = m.scope_id)
        GROUP BY replace(replace(lower(trim(m.agent_key)),'-','_'),' ','_');

        INSERT OR IGNORE INTO memory_banks
            (id, kind, scope_id, run_id, owner_task_id, parent_bank_id,
             display_name, rev, created_at, closed_at)
        SELECT 'bank:branch:'||t.id, 'branch', t.id, t.run_id, t.id,
               'bank:run:'||t.run_id, 'Task '||substr(t.id,1,8), 0,
               MIN(m.created_at), NULL
        FROM agent_memory m
        JOIN tasks t ON t.id = m.scope_id
        GROUP BY t.id;

        INSERT OR IGNORE INTO memory_entries
            (id, bank_id, entry_key, kind, value, source_kind, source_task_id,
             source_run_id, trust, status, created_at, updated_at)
        SELECT 'entry:legacy:'||m.rowid,
               'bank:agent_role:'||
               replace(replace(lower(trim(m.agent_key)),'-','_'),' ','_'),
               'legacy:'||m.scope_id||':'||m.agent_key||':'||m.key,
               'note', m.value, 'migration', m.task_id, m.run_id,
               NULL, 'active', m.created_at, m.created_at
        FROM agent_memory m
        WHERE m.scope_id = 'legacy'
           OR NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = m.scope_id);

        INSERT OR IGNORE INTO memory_entries
            (id, bank_id, entry_key, kind, value, source_kind, source_task_id,
             source_run_id, trust, status, created_at, updated_at)
        SELECT 'entry:run:'||m.rowid, 'bank:run:'||m.run_id,
               'legacy:run:'||m.agent_key||':'||m.key, 'note', m.value,
               'migration', m.task_id, m.run_id, NULL, 'active',
               m.created_at, m.created_at
        FROM agent_memory m
        WHERE m.scope_id = 'run'
          AND EXISTS (SELECT 1 FROM runs r WHERE r.id = m.run_id);

        INSERT OR IGNORE INTO memory_entries
            (id, bank_id, entry_key, kind, value, source_kind, source_task_id,
             source_run_id, trust, status, created_at, updated_at)
        SELECT 'entry:branch:'||m.rowid, 'bank:branch:'||t.id,
               'legacy:branch:'||m.agent_key||':'||m.key, 'note', m.value,
               'migration', m.task_id, m.run_id, NULL, 'active',
               m.created_at, m.created_at
        FROM agent_memory m JOIN tasks t ON t.id = m.scope_id;
    "#,
    down: r#"
        DROP TABLE IF EXISTS memory_entries;
        DROP TABLE IF EXISTS memory_banks;
    "#,
},
Migration {
    version: 15,
    name: "memory_journal",
    up: r#"
        CREATE TABLE memory_journal (
            seq         INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id      TEXT,
            bank_id     TEXT,
            entry_id    TEXT,
            action      TEXT NOT NULL,
            actor_kind  TEXT NOT NULL,
            actor_id    TEXT,
            detail_json TEXT NOT NULL DEFAULT '{}',
            created_at  TEXT NOT NULL
        );
        CREATE INDEX idx_memory_journal_run ON memory_journal(run_id, seq);
        CREATE INDEX idx_memory_journal_bank ON memory_journal(bank_id, seq);
        INSERT INTO memory_journal
            (run_id, bank_id, entry_id, action, actor_kind, actor_id,
             detail_json, created_at)
        SELECT e.source_run_id, e.bank_id, e.id, 'entry_written', 'system',
               NULL, '{"source":"migration_v15"}', e.created_at
        FROM memory_entries e
        WHERE e.source_kind = 'migration';
    "#,
    down: r#"
        DROP TABLE IF EXISTS memory_journal;
    "#,
},
Migration {
    version: 16,
    name: "memory_context_snapshots",
    up: r#"
        CREATE TABLE memory_context_snapshots (
            id             TEXT PRIMARY KEY NOT NULL,
            task_id        TEXT NOT NULL,
            run_id         TEXT NOT NULL,
            banks_json     TEXT NOT NULL,
            entry_ids_json TEXT NOT NULL DEFAULT '[]',
            rendered_chars INTEGER NOT NULL CHECK (rendered_chars >= 0),
            truncated      INTEGER NOT NULL CHECK (truncated IN (0,1)),
            prompt_sha256  TEXT NOT NULL,
            created_at     TEXT NOT NULL
        );
        CREATE INDEX idx_memory_snapshots_task ON memory_context_snapshots(task_id);
        CREATE INDEX idx_memory_snapshots_run ON memory_context_snapshots(run_id);
    "#,
    down: r#"
        DROP TABLE IF EXISTS memory_context_snapshots;
    "#,
},
Migration {
    version: 17,
    name: "task_ownership",
    up: r#"
        ALTER TABLE tasks ADD COLUMN ownership_mode TEXT;
        ALTER TABLE tasks ADD COLUMN ownership_scope TEXT;
        ALTER TABLE tasks ADD COLUMN owner_task_id TEXT;
        UPDATE tasks SET ownership_mode = 'run',
            ownership_scope = 'run:'||run_id, owner_task_id = id
        WHERE class = 'hyper';
        UPDATE tasks SET ownership_mode = 'branch',
            ownership_scope = 'hive:'||id, owner_task_id = id
        WHERE class = 'super';
        UPDATE tasks SET ownership_mode = 'none'
        WHERE class IN ('agent','subagent') AND ownership_mode IS NULL;
        CREATE TRIGGER IF NOT EXISTS tasks_ownership_after_insert
        AFTER INSERT ON tasks
        WHEN NEW.ownership_mode IS NULL
        BEGIN
            UPDATE tasks SET
                ownership_mode = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'run'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN 'branch'
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('agent','worker','workeragent','worker-agent','content-writer',
                         'data-analyst','code-reviewer','chatbot','subagent','sub-agent') THEN 'none'
                    ELSE NULL END,
                ownership_scope = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator') THEN 'run:'||NEW.run_id
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN 'hive:'||NEW.id
                    ELSE NULL END,
                owner_task_id = CASE
                    WHEN replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('planner','hyper','hyperagent','hyper-agent','orchestrator')
                     OR replace(replace(lower(trim(NEW.role)),'_','-'),' ','-') IN
                        ('super','superagent','super-agent','sub-hive','subhive') THEN NEW.id
                    ELSE NULL END
            WHERE id = NEW.id;
        END;
        CREATE INDEX IF NOT EXISTS idx_tasks_owner ON tasks(run_id, owner_task_id);
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_tasks_owner;
        DROP TRIGGER IF EXISTS tasks_ownership_after_insert;
        ALTER TABLE tasks DROP COLUMN owner_task_id;
        ALTER TABLE tasks DROP COLUMN ownership_scope;
        ALTER TABLE tasks DROP COLUMN ownership_mode;
    "#,
},
Migration {
    version: 18,
    name: "memory_journal_hash_chain",
    up: r#"
        ALTER TABLE memory_journal ADD COLUMN prev_hash TEXT;
        ALTER TABLE memory_journal ADD COLUMN event_hash TEXT;
        CREATE INDEX idx_memory_journal_hash ON memory_journal(run_id, event_hash);
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_memory_journal_hash;
        ALTER TABLE memory_journal DROP COLUMN event_hash;
        ALTER TABLE memory_journal DROP COLUMN prev_hash;
    "#,
},
Migration {
    version: 19,
    name: "single_run_owner",
    up: r#"
        CREATE UNIQUE INDEX IF NOT EXISTS idx_tasks_single_run_owner
        ON tasks(run_id) WHERE ownership_mode = 'run';
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_tasks_single_run_owner;
    "#,
},
Migration {
    version: 20,
    name: "memory_journal_head",
    up: r#"
        ALTER TABLE runs ADD COLUMN memory_journal_head TEXT;
        UPDATE runs SET memory_journal_head = (
            SELECT event_hash FROM memory_journal j
            WHERE j.run_id = runs.id AND j.event_hash IS NOT NULL
            ORDER BY j.seq DESC LIMIT 1
        );
    "#,
    down: r#"
        ALTER TABLE runs DROP COLUMN memory_journal_head;
    "#,
},
Migration {
    version: 21,
    name: "remote_control_plane",
    up: r#"
        CREATE TABLE remote_agents (
            remote_id         TEXT PRIMARY KEY NOT NULL,
            display_name      TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 120),
            owner_kind        TEXT NOT NULL CHECK (owner_kind IN ('operator','hive','office')),
            owner_id          TEXT NOT NULL,
            transport_kind    TEXT NOT NULL CHECK (transport_kind IN ('in_process','lan_http','mqtt_tls','outbound_ws')),
            adapter_kind      TEXT NOT NULL,
            endpoint_ref      TEXT NOT NULL,
            credential_ref    TEXT,
            capabilities_json TEXT NOT NULL DEFAULT '[]',
            state             TEXT NOT NULL DEFAULT 'enrolled'
                              CHECK (state IN ('enrolled','online','degraded','offline','quarantined','revoked','retired')),
            generation        INTEGER NOT NULL DEFAULT 1,
            rev               INTEGER NOT NULL DEFAULT 0,
            restart_attempts  INTEGER NOT NULL DEFAULT 0,
            restart_window_at TEXT,
            last_seen_at      TEXT,
            lease_expires_at  TEXT,
            quarantined_at    TEXT,
            revoked_at        TEXT,
            created_by        TEXT NOT NULL,
            created_at        TEXT NOT NULL
        );
        CREATE INDEX idx_remote_state ON remote_agents(state, last_seen_at);
        CREATE UNIQUE INDEX ux_remote_name ON remote_agents(display_name) WHERE revoked_at IS NULL;

        CREATE TABLE devices (
            device_id         TEXT PRIMARY KEY NOT NULL,
            remote_id         TEXT NOT NULL REFERENCES remote_agents(remote_id),
            class             TEXT NOT NULL CHECK (class IN ('light','switch','camera','cover','lock','sensor','other')),
            name              TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 120),
            room              TEXT,
            adapter_ref       TEXT NOT NULL,
            capabilities_json TEXT NOT NULL,
            state_json        TEXT NOT NULL DEFAULT '{}',
            state_version     INTEGER NOT NULL DEFAULT 0,
            last_seen_at      TEXT,
            safety_class      TEXT NOT NULL CHECK (safety_class IN ('read_only','reversible','critical')),
            revoked_at        TEXT,
            created_at        TEXT NOT NULL
        );
        CREATE INDEX idx_devices_remote ON devices(remote_id, class);
        CREATE UNIQUE INDEX ux_device_remote_ref ON devices(remote_id, adapter_ref) WHERE revoked_at IS NULL;

        CREATE TABLE remote_grants (
            grant_id         TEXT PRIMARY KEY NOT NULL,
            remote_id        TEXT NOT NULL REFERENCES remote_agents(remote_id),
            device_id        TEXT NOT NULL REFERENCES devices(device_id),
            capability_id    TEXT NOT NULL,
            action           TEXT NOT NULL,
            constraints_json TEXT NOT NULL DEFAULT '{}',
            granted_by       TEXT NOT NULL,
            granted_at       TEXT NOT NULL,
            expires_at       TEXT,
            revoked_at       TEXT,
            rev              INTEGER NOT NULL DEFAULT 0
        );
        CREATE UNIQUE INDEX ux_remote_grant ON remote_grants(remote_id, device_id, capability_id, action)
            WHERE revoked_at IS NULL;

        CREATE TABLE device_actions (
            action_id          TEXT PRIMARY KEY NOT NULL,
            remote_id          TEXT NOT NULL REFERENCES remote_agents(remote_id),
            device_id          TEXT NOT NULL REFERENCES devices(device_id),
            capability_id      TEXT NOT NULL,
            action             TEXT NOT NULL,
            params_json        TEXT NOT NULL,
            params_sha256      TEXT NOT NULL,
            expected_json      TEXT,
            request_id         TEXT NOT NULL,
            idempotency_key    TEXT NOT NULL,
            requested_by_kind  TEXT NOT NULL CHECK (requested_by_kind IN ('operator','task','remote')),
            requested_by_id    TEXT NOT NULL,
            approval_nonce     TEXT,
            approval_expires_at TEXT,
            status             TEXT NOT NULL CHECK (status IN ('requested','awaiting_approval','approved','denied','executing','confirmed','failed','timeout','unknown','cancelled','interrupted')),
            state_before_hash  TEXT,
            state_after_hash   TEXT,
            error_code         TEXT,
            created_at         TEXT NOT NULL,
            started_at         TEXT,
            ended_at           TEXT,
            UNIQUE(remote_id, device_id, idempotency_key)
        );
        CREATE INDEX idx_device_actions_status ON device_actions(status, created_at);
        CREATE INDEX idx_device_actions_device ON device_actions(device_id, created_at);

        CREATE TABLE remote_audit (
            audit_id        TEXT PRIMARY KEY NOT NULL,
            at              TEXT NOT NULL,
            actor_kind      TEXT NOT NULL CHECK (actor_kind IN ('operator','system','remote')),
            actor_id        TEXT NOT NULL,
            remote_id       TEXT,
            device_id       TEXT,
            action          TEXT NOT NULL,
            request_id      TEXT,
            idempotency_key TEXT,
            params_sha256   TEXT,
            outcome         TEXT NOT NULL,
            refusal_code    TEXT,
            detail_json     TEXT NOT NULL DEFAULT '{}',
            created_at      TEXT NOT NULL
        );
        CREATE INDEX idx_remote_audit_remote ON remote_audit(remote_id, at);
        CREATE INDEX idx_remote_audit_device ON remote_audit(device_id, at);

        CREATE TABLE remote_supervisor_state (
            id              INTEGER PRIMARY KEY CHECK (id = 1),
            lane_started_at TEXT,
            last_tick_at    TEXT,
            next_due_at     TEXT,
            backoff_ms      INTEGER NOT NULL DEFAULT 0,
            last_error_code TEXT,
            updated_at      TEXT NOT NULL
        );
        INSERT INTO remote_supervisor_state
            (id, lane_started_at, last_tick_at, next_due_at, backoff_ms, last_error_code, updated_at)
        VALUES (1, NULL, NULL, NULL, 0, NULL, '1970-01-01T00:00:00Z');
    "#,
    down: r#"
        DROP TABLE IF EXISTS remote_supervisor_state;
        DROP TABLE IF EXISTS remote_audit;
        DROP TABLE IF EXISTS device_actions;
        DROP TABLE IF EXISTS remote_grants;
        DROP TABLE IF EXISTS devices;
        DROP TABLE IF EXISTS remote_agents;
    "#,
},
Migration {
    version: 22,
    name: "daemon_plane",
    up: r#"
        CREATE TABLE daemons (
            daemon_id         TEXT PRIMARY KEY NOT NULL,
            display_name      TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 120),
            owner_kind        TEXT NOT NULL CHECK (owner_kind IN ('operator','hive','remote')),
            owner_id          TEXT NOT NULL,
            kind              TEXT NOT NULL CHECK (kind IN ('watchdog','relay','switch','satellite')),
            job_json          TEXT NOT NULL DEFAULT '{}',
            state             TEXT NOT NULL DEFAULT 'enrolled'
                              CHECK (state IN ('enrolled','running','degraded','stopped','retired')),
            interval_secs     INTEGER NOT NULL CHECK (interval_secs BETWEEN 5 AND 86400),
            next_tick_at      TEXT,
            last_tick_at      TEXT,
            last_report_at    TEXT,
            restart_attempts  INTEGER NOT NULL DEFAULT 0,
            restart_window_at TEXT,
            last_error_code   TEXT,
            created_by        TEXT NOT NULL,
            created_at        TEXT NOT NULL,
            retired_at        TEXT
        );
        CREATE INDEX idx_daemons_state ON daemons(state, next_tick_at);
        CREATE INDEX idx_daemons_owner ON daemons(owner_kind, owner_id);

        CREATE TABLE daemon_reports (
            report_id    TEXT PRIMARY KEY NOT NULL,
            daemon_id    TEXT NOT NULL REFERENCES daemons(daemon_id) ON DELETE CASCADE,
            outcome      TEXT NOT NULL,
            detail_json  TEXT NOT NULL DEFAULT '{}',
            created_at   TEXT NOT NULL
        );
        CREATE INDEX idx_daemon_reports_daemon ON daemon_reports(daemon_id, created_at);
    "#,
    down: r#"
        DROP TABLE IF EXISTS daemon_reports;
        DROP TABLE IF EXISTS daemons;
    "#,
},
Migration {
    version: 23,
    name: "a2a_attempt_audit",
    up: r#"
        ALTER TABLE agent_messages ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0;
        ALTER TABLE agent_messages ADD COLUMN last_attempt_at TEXT;
        CREATE TABLE a2a_audit (
            audit_id    TEXT PRIMARY KEY NOT NULL,
            run_id      TEXT NOT NULL,
            message_id  TEXT NOT NULL,
            actor       TEXT NOT NULL CHECK (length(actor) BETWEEN 1 AND 160),
            action      TEXT NOT NULL CHECK (length(action) BETWEEN 1 AND 64),
            detail_json TEXT NOT NULL DEFAULT '{}',
            created_at  TEXT NOT NULL
        );
        CREATE INDEX idx_a2a_audit_run ON a2a_audit(run_id, created_at);
        CREATE INDEX idx_a2a_audit_message ON a2a_audit(message_id, created_at);
    "#,
    down: r#"
        DROP INDEX IF EXISTS idx_a2a_audit_message;
        DROP INDEX IF EXISTS idx_a2a_audit_run;
        DROP TABLE IF EXISTS a2a_audit;
        ALTER TABLE agent_messages DROP COLUMN last_attempt_at;
        ALTER TABLE agent_messages DROP COLUMN attempts;
    "#,
},
Migration {
    version: 24,
    name: "task_instruction",
    up: r#"
        ALTER TABLE tasks ADD COLUMN instruction TEXT;
    "#,
    down: r#"
        ALTER TABLE tasks DROP COLUMN instruction;
    "#,
},
Migration {
    version: 25,
    name: "sessions_policy_todos",
    up: r#"
        CREATE TABLE sessions (
            id                TEXT PRIMARY KEY NOT NULL,
            project_id        TEXT NOT NULL,
            name              TEXT NOT NULL,
            workspace_root    TEXT NOT NULL,
            permission_policy TEXT NOT NULL DEFAULT 'classic'
                              CHECK (permission_policy IN ('classic','plan','standard','trusted')),
            created_at        TEXT NOT NULL,
            updated_at        TEXT NOT NULL
        );
        CREATE INDEX idx_sessions_project ON sessions(project_id, created_at);
        CREATE TABLE session_events (
            id         TEXT PRIMARY KEY NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            kind       TEXT NOT NULL,
            detail_json TEXT NOT NULL DEFAULT '{}',
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_session_events_session ON session_events(session_id, created_at);
        CREATE TABLE todos (
            id         TEXT PRIMARY KEY NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            seq        INTEGER NOT NULL,
            text       TEXT NOT NULL,
            state      TEXT NOT NULL DEFAULT 'open'
                       CHECK (state IN ('open','done','cancelled')),
            created_at TEXT NOT NULL
        );
        CREATE INDEX idx_todos_session ON todos(session_id, seq);
        ALTER TABLE runs ADD COLUMN session_id TEXT;
        ALTER TABLE runs ADD COLUMN permission_policy TEXT NOT NULL DEFAULT 'classic'
            CHECK (permission_policy IN ('classic','plan','standard','trusted'));
    "#,
    down: r#"
        ALTER TABLE runs DROP COLUMN permission_policy;
        ALTER TABLE runs DROP COLUMN session_id;
        DROP INDEX IF EXISTS idx_todos_session;
        DROP TABLE IF EXISTS todos;
        DROP INDEX IF EXISTS idx_session_events_session;
        DROP TABLE IF EXISTS session_events;
        DROP INDEX IF EXISTS idx_sessions_project;
        DROP TABLE IF EXISTS sessions;
    "#,
},
Migration {
    version: 26,
    name: "repo_index",
    up: crate::repo_index::SCHEMA_V26,
    down: crate::repo_index::SCHEMA_V26_DOWN,
},
crate::checkpoint::checkpoint_migration(27),
crate::jobs::job_migration(28),
crate::eval::eval_migration(29),
crate::learning::learning_migration(30),
crate::eval::eval_suite_migration(31),
Migration {
    version: 32,
    name: "session_archive",
    up: "ALTER TABLE sessions ADD COLUMN archived_at TEXT;",
    down: "ALTER TABLE sessions DROP COLUMN archived_at;",
},
];
/// Database errors; each maps to a named refusal for the wire.
#[derive(Debug, Error)]
pub enum DbError {
    /// SQLite reported lock contention (`E_DB_BUSY`, retryable).
    #[error("E_DB_BUSY: the local SQLite database is locked: {0}")]
    Busy(String),
    /// Any other SQLite error.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// Filesystem error while opening/creating the database.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// Another thread panicked while holding the connection lock.
    #[error("database connection lock is poisoned")]
    Poisoned,
    /// The requested down-migration target is not a known version.
    #[error("unknown migration version {0}")]
    UnknownMigration(i64),
    /// The database was written by a newer schema than this binary supports.
    #[error("database schema {current} is newer than the newest supported migration {latest}")]
    SchemaNewer {
        /// Schema version found in the database.
        current: i64,
        /// Newest migration compiled into this binary.
        latest: i64,
    },
    /// The pre-migration backup was created but failed integrity verification.
    #[error("pre-migration backup failed integrity check: {0}")]
    BackupInvalid(String),
    /// A tool-turn row could not be stored or read truthfully (integrity defect,
    /// not a transport failure).
    #[error("invalid tool-turn persistence: {0}")]
    InvalidToolTurn(String),
}

impl DbError {
    /// Maps the error to the named refusal used on the wire.
    #[must_use]
    pub fn refusal(&self) -> Refusal {
        match self {
            DbError::Busy(_) => Refusal::DbBusy,
            DbError::SchemaNewer { .. } => Refusal::SchemaNewer,
            DbError::Sqlite(rusqlite::Error::SqliteFailure(code, _))
                if matches!(
                    code.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) =>
            {
                Refusal::DbBusy
            }
            _ => Refusal::InternalError,
        }
    }
}

/// The local SQLite database.
pub struct Database {
    path: PathBuf,
    connection: Mutex<Connection>,
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Database").field("path", &self.path).finish()
    }
}

impl Database {
    /// Opens (creating if needed) the database file and applies pragmas.
    ///
    /// # Errors
    /// [`DbError`] when the file cannot be opened or configured.
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let connection = Connection::open(path)?;
        Self::configure(&connection)?;
        Ok(Self {
            path: path.to_path_buf(),
            connection: Mutex::new(connection),
        })
    }

    /// Opens a fresh in-memory database (tests and throwaway sessions only).
    ///
    /// # Errors
    /// [`DbError`] when the in-memory database cannot be configured.
    pub fn open_in_memory() -> Result<Self, DbError> {
        let connection = Connection::open_in_memory()?;
        Self::configure(&connection)?;
        Ok(Self {
            path: PathBuf::from(":memory:"),
            connection: Mutex::new(connection),
        })
    }

    fn configure(connection: &Connection) -> Result<(), DbError> {
        connection.busy_timeout(std::time::Duration::from_millis(5_000))?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        Ok(())
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, DbError> {
        self.connection.lock().map_err(|_| DbError::Poisoned)
    }

    /// Path of the database file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Current applied schema version (`0` for a fresh database).
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn schema_version(&self) -> Result<i64, DbError> {
        let connection = self.lock()?;
        Self::current_version(&connection)
    }

    fn current_version(connection: &Connection) -> Result<i64, DbError> {
        if !Self::table_exists(connection, "schema_migrations")? {
            return Ok(0);
        }
        let version: Option<i64> =
            connection.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })?;
        Ok(version.unwrap_or(0))
    }

    fn table_exists(connection: &Connection, name: &str) -> Result<bool, DbError> {
        let tables: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        )?;
        Ok(tables > 0)
    }

    /// Applies every pending migration in order (idempotent).
    ///
    /// # Errors
    /// [`DbError`] when a migration fails; the failed migration is not recorded.
    pub fn migrate(&self) -> Result<i64, DbError> {
        let mut connection = self.lock()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        let current = Self::current_version(&connection)?;
        let latest = MIGRATIONS.last().map_or(0, |migration| migration.version);
        if current > latest {
            return Err(DbError::SchemaNewer { current, latest });
        }
        if current > 0 && MIGRATIONS.iter().any(|migration| migration.version > current) {
            self.backup_before_migration(&connection, current)?;
        }
        for migration in MIGRATIONS {
            if migration.version <= current {
                continue;
            }
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.up)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                rusqlite::params![
                    migration.version,
                    migration.name,
                    Timestamp::now().as_str()
                ],
            )?;
            transaction.commit()?;
        }
        Self::current_version(&connection)
    }

    /// Writes and verifies a consistent backup before any pending migration.
    ///
    /// The backup is created with SQLite `VACUUM INTO` (safe with WAL) and is
    /// verified with `PRAGMA integrity_check`; a failed backup refuses migration.
    fn backup_before_migration(
        &self,
        connection: &Connection,
        version: i64,
    ) -> Result<PathBuf, DbError> {
        if self.path == Path::new(":memory:") {
            return Ok(PathBuf::new());
        }
        let stem = self
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("hiverouter");
        let stamp: String = Timestamp::now()
            .as_str()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character
                } else {
                    '-'
                }
            })
            .collect();
        let mut backup = self
            .path
            .with_file_name(format!("{stem}.pre-v{version}-{stamp}.bak"));
        let mut suffix = 1_u32;
        while backup.exists() {
            backup = self
                .path
                .with_file_name(format!("{stem}.pre-v{version}-{stamp}-{suffix}.bak"));
            suffix = suffix.saturating_add(1);
        }
        connection.execute(
            "VACUUM INTO ?1",
            rusqlite::params![backup.to_string_lossy().to_string()],
        )?;
        let verified = Connection::open(&backup)?;
        let integrity: String =
            verified.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(DbError::BackupInvalid(format!(
                "{}: {integrity}",
                backup.display()
            )));
        }
        Ok(backup)
    }

    /// Rolls back migrations down to (and including) `target` being the highest
    /// version kept. Used by the migration-roundtrip test.
    ///
    /// # Errors
    /// [`DbError`] when a down migration fails or `target` is unknown.
    pub fn migrate_down_to(&self, target: i64) -> Result<i64, DbError> {
        if target != 0 && !MIGRATIONS.iter().any(|m| m.version == target) {
            return Err(DbError::UnknownMigration(target));
        }
        let mut connection = self.lock()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version    INTEGER PRIMARY KEY,
                name       TEXT NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;
        loop {
            let current = Self::current_version(&connection)?;
            if current <= target {
                return Ok(current);
            }
            let migration = MIGRATIONS
                .iter()
                .find(|m| m.version == current)
                .ok_or(DbError::UnknownMigration(current))?;
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.down)?;
            transaction.execute(
                "DELETE FROM schema_migrations WHERE version = ?1",
                rusqlite::params![migration.version],
            )?;
            transaction.commit()?;
        }
    }

    /// Applied migrations as `(version, name, applied_at)`.
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn applied_migrations(&self) -> Result<Vec<(i64, String, String)>, DbError> {
        let connection = self.lock()?;
        if !Self::table_exists(&connection, "schema_migrations")? {
            return Ok(Vec::new());
        }
        let mut statement =
            connection.prepare("SELECT version, name, applied_at FROM schema_migrations ORDER BY version")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })?;
        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(row?);
        }
        Ok(migrations)
    }

    /// Reads one `app_meta` value.
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn app_meta(&self, key: &str) -> Result<Option<String>, DbError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT value FROM app_meta WHERE key = ?1")?;
        let mut rows = statement.query(rusqlite::params![key])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    /// Runs a closure with exclusive access to the connection.
    ///
    /// # Errors
    /// [`DbError`] when the lock is poisoned or the closure returns a SQLite
    /// error.
    pub fn with_connection<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> rusqlite::Result<T>,
    ) -> Result<T, DbError> {
        let mut connection = self.lock()?;
        Ok(f(&mut connection)?)
    }

    /// Runs a closure that reports named refusals directly (tools-layer calls).
    ///
    /// This keeps a business refusal such as `E_APPROVAL_REPLAY` from being
    /// flattened into a generic database error by a transport wrapper.
    ///
    /// # Errors
    /// The closure's named refusal, or `E_INTERNAL_ERROR` when the lock is
    /// poisoned.
    pub fn with_connection_refusal<T>(
        &self,
        f: impl FnOnce(&mut Connection) -> Result<T, Refusal>,
    ) -> Result<T, Refusal> {
        let mut connection = self.lock().map_err(|_| Refusal::InternalError)?;
        f(&mut connection)
    }

    /// Persists one tool-call record for a run (1.2 tool loop, schema v4).
    ///
    /// `UNIQUE(run_id, tool_call_id)` makes a model tool call single-effect:
    /// recording the same provider tool-call id twice fails with a duplicate
    /// constraint error instead of silently duplicating side-effect history.
    ///
    /// # Errors
    /// [`DbError::Sqlite`] on SQLite failure (including the uniqueness
    /// constraint), [`DbError::InvalidToolTurn`] when a value cannot be stored
    /// truthfully (for example a byte count outside the SQLite INTEGER range),
    /// and [`DbError::Poisoned`] when the connection lock is poisoned.
    pub fn record_tool_turn(&self, turn: &ToolTurnRecord) -> Result<(), DbError> {
        let bytes = tool_turn_bytes_to_sql(turn.bytes)?;
        let id = turn.id.clone();
        let run_id = turn.run_id.to_string();
        let task_id = turn.task_id.to_string();
        let tool_call_id = turn.tool_call_id.clone();
        let tool = turn.tool.clone();
        let args_sha256 = turn.args_sha256.clone();
        let outcome = turn.outcome.as_str();
        let code = turn.code.clone();
        let sha256 = turn.sha256.clone();
        let turn_number = i64::from(turn.turn);
        let created_at = turn.created_at.as_str().to_owned();
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO tool_turns(
                 id, run_id, task_id, tool_call_id, tool, args_sha256,
                 outcome, code, bytes, sha256, turn, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                id,
                run_id,
                task_id,
                tool_call_id,
                tool,
                args_sha256,
                outcome,
                code,
                bytes,
                sha256,
                turn_number,
                created_at
            ],
        )?;
        Ok(())
    }

    /// Reads one task's completed tool turns (the per-task turn cap source).
    ///
    /// # Errors
    /// [`DbError`] when the query fails.
    pub fn task_tool_turns(&self, run_id: RunId, task_id: TaskId) -> Result<u32, DbError> {
        let connection = self.lock()?;
        let turns: i64 = connection.query_row(
            "SELECT COUNT(DISTINCT turn) FROM tool_turns WHERE run_id = ?1 AND task_id = ?2",
            rusqlite::params![run_id.to_string(), task_id.to_string()],
            |row| row.get(0),
        )?;
        u32::try_from(turns)
            .map_err(|_| DbError::InvalidToolTurn(format!("negative task turn count {turns}")))
    }

    /// Reads one run's persisted tool turns, ordered by turn then creation time.
    ///
    /// # Errors
    /// [`DbError`] when the query fails or a stored row cannot be decoded
    /// truthfully (invalid id/timestamp/outcome/negative byte count).
    pub fn tool_turns_for_run(&self, run_id: RunId) -> Result<Vec<ToolTurnRecord>, DbError> {
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT id, run_id, task_id, tool_call_id, tool, args_sha256,
                    outcome, code, bytes, sha256, turn, created_at
             FROM tool_turns
             WHERE run_id = ?1
             ORDER BY turn, created_at, id",
        )?;
        let rows = statement.query_map(rusqlite::params![run_id.to_string()], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<i64>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, i64>(10)?,
                row.get::<_, String>(11)?,
            ))
        })?;
        let mut records = Vec::new();
        for row in rows {
            let (
                id,
                run_raw,
                task_raw,
                tool_call_id,
                tool,
                args_sha256,
                outcome_raw,
                code,
                bytes,
                sha256,
                turn_raw,
                created_at_raw,
            ) = row?;
            let parsed_run = run_raw.parse::<RunId>().map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid run_id `{run_raw}` in tool_turns"))
            })?;
            let parsed_task = task_raw.parse::<TaskId>().map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid task_id `{task_raw}` in tool_turns"))
            })?;
            let parsed_outcome = ToolOutcome::from_str(&outcome_raw).ok_or_else(|| {
                DbError::InvalidToolTurn(format!(
                    "unknown tool outcome `{outcome_raw}` in tool_turns"
                ))
            })?;
            let parsed_bytes = tool_turn_bytes_from_sql(bytes)?;
            let parsed_turn = u32::try_from(turn_raw).map_err(|_| {
                DbError::InvalidToolTurn(format!("invalid turn {turn_raw} in tool_turns"))
            })?;
            let created_at = Timestamp::parse(&created_at_raw).map_err(|_| {
                DbError::InvalidToolTurn(format!(
                    "invalid created_at `{created_at_raw}` in tool_turns"
                ))
            })?;
            records.push(ToolTurnRecord {
                id,
                run_id: parsed_run,
                task_id: parsed_task,
                tool_call_id,
                tool,
                args_sha256,
                outcome: parsed_outcome,
                code,
                bytes: parsed_bytes,
                sha256,
                turn: parsed_turn,
                created_at,
            });
        }
        Ok(records)
    }

    /// Adds to the per-run tool counters (`runs.tool_turns`, `runs.tool_calls`).
    ///
    /// # Errors
    /// [`DbError::InvalidToolTurn`] when the run does not exist, plus the usual
    /// SQLite/lock errors. A missing run is never reported as `(0, 0)`.
    pub fn increment_run_tool_counters(
        &self,
        run_id: RunId,
        turns: u32,
        calls: u32,
    ) -> Result<(), DbError> {
        let connection = self.lock()?;
        let changed = connection.execute(
            "UPDATE runs
             SET tool_turns = tool_turns + ?2, tool_calls = tool_calls + ?3
             WHERE id = ?1",
            rusqlite::params![
                run_id.to_string(),
                i64::from(turns),
                i64::from(calls)
            ],
        )?;
        if changed != 1 {
            return Err(DbError::InvalidToolTurn(format!(
                "run `{run_id}` not found while incrementing tool counters"
            )));
        }
        Ok(())
    }

    /// Reads the per-run tool counters as `(tool_turns, tool_calls)`.
    ///
    /// # Errors
    /// [`DbError::InvalidToolTurn`] when the run does not exist or a stored
    /// counter is not a non-negative integer; never fabricates `(0, 0)`.
    pub fn run_tool_counters(&self, run_id: RunId) -> Result<(u32, u32), DbError> {
        let connection = self.lock()?;
        let row: Option<(i64, i64)> = connection
            .query_row(
                "SELECT tool_turns, tool_calls FROM runs WHERE id = ?1",
                rusqlite::params![run_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (turns_raw, calls_raw) = row.ok_or_else(|| {
            DbError::InvalidToolTurn(format!("run `{run_id}` not found while reading tool counters"))
        })?;
        let turns = u32::try_from(turns_raw).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative tool_turns {turns_raw} for run {run_id}"))
        })?;
        let calls = u32::try_from(calls_raw).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative tool_calls {calls_raw} for run {run_id}"))
        })?;
        Ok((turns, calls))
    }
}

fn tool_turn_bytes_to_sql(bytes: Option<u64>) -> Result<Option<i64>, DbError> {
    match bytes {
        Some(value) => i64::try_from(value).map(Some).map_err(|_| {
            DbError::InvalidToolTurn(format!("bytes {value} exceeds the SQLite INTEGER range"))
        }),
        None => Ok(None),
    }
}

fn tool_turn_bytes_from_sql(bytes: Option<i64>) -> Result<Option<u64>, DbError> {
    match bytes {
        Some(value) => u64::try_from(value).map(Some).map_err(|_| {
            DbError::InvalidToolTurn(format!("negative byte count {value} in tool_turns"))
        }),
        None => Ok(None),
    }
}

/// One persisted tool call in a run (schema v4, 1.2 tool loop).
///
/// Mirrors the `tool_turns` row exactly: `args_sha256` is the only argument
/// representation stored (never the raw arguments), and nullable `code`/`bytes`/
/// `sha256` stay absent rather than being coerced to zero/empty.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTurnRecord {
    /// Row id (uuid-v7 string minted by the caller).
    pub id: String,
    /// Run the tool call belongs to.
    pub run_id: RunId,
    /// Task that issued the tool call.
    pub task_id: TaskId,
    /// Provider tool-call id (unique per run).
    pub tool_call_id: String,
    /// Tool id from the frozen tool registry.
    pub tool: String,
    /// SHA-256 (lowercase hex) of the canonical JSON arguments.
    pub args_sha256: String,
    /// Tool-call outcome vocabulary (`ok|refused|denied|timeout|cancelled`).
    pub outcome: ToolOutcome,
    /// Named refusal code when the outcome carries one.
    pub code: Option<String>,
    /// Result size in bytes when known; absent stays absent.
    pub bytes: Option<u64>,
    /// SHA-256 of the result body when known.
    pub sha256: Option<String>,
    /// Loop turn (1-based) the call belongs to.
    pub turn: u32,
    /// Row creation timestamp (RFC3339 UTC).
    pub created_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hiverouter_contract::SCHEMA_VERSION;

    #[test]
    fn fresh_database_is_version_zero() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.schema_version().expect("version"), 0);
        assert!(database.applied_migrations().expect("migrations").is_empty());
    }

    #[test]
    fn migrate_is_idempotent_and_records_bootstrap() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.migrate().expect("first migrate"), 32);
        assert_eq!(database.migrate().expect("second migrate"), 32);
        let applied = database.applied_migrations().expect("migrations");
        assert_eq!(applied.len(), 32);
        assert_eq!(applied[0].0, 1);
        assert_eq!(applied[0].1, "bootstrap");
        assert_eq!(applied[1].0, 2);
        assert_eq!(applied[1].1, "runs_tasks_chat");
        assert_eq!(applied[2].0, 3);
        assert_eq!(applied[2].1, "snapshot_index");
        assert_eq!(applied[3].0, 4);
        assert_eq!(applied[3].1, "tool_turns");
        assert_eq!(applied[4].0, 5);
        assert_eq!(applied[4].1, "task_results");
        assert_eq!(applied[5].0, 6);
        assert_eq!(applied[5].1, "agent_messages");
        assert_eq!(applied[6].0, 7);
        assert_eq!(applied[6].1, "agent_memory");
        assert_eq!(applied[7].0, 8);
        assert_eq!(applied[7].1, "run_usage");
        assert_eq!(applied[8].0, 9);
        assert_eq!(applied[8].1, "agent_profiles");
        assert_eq!(applied[9].0, 10);
        assert_eq!(applied[9].1, "projects");
        assert_eq!(applied[10].0, 11);
        assert_eq!(applied[10].1, "agent_memory_scoped");
        assert_eq!(
            database.app_meta("schema_version").expect("meta").as_deref(),
            Some(SCHEMA_VERSION)
        );
        assert!(database.app_meta("bootstrapped_at").expect("meta").is_some());
    }

    #[test]
    fn down_then_up_restores_the_schema() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        assert_eq!(database.migrate_down_to(0).expect("down"), 0);
        assert_eq!(database.schema_version().expect("version"), 0);
        assert!(database.app_meta("schema_version").is_err());
        assert_eq!(database.migrate().expect("re-migrate"), 32);
        assert_eq!(
            database.app_meta("schema_version").expect("meta").as_deref(),
            Some(SCHEMA_VERSION)
        );
    }

    #[test]
    fn file_database_persists_across_handles() {
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("hiverouter.db");
        {
            let database = Database::open(&path).expect("open");
            database.migrate().expect("migrate");
        }
        let reopened = Database::open(&path).expect("reopen");
        assert_eq!(reopened.schema_version().expect("version"), 32);
        assert_eq!(reopened.path(), path);
    }

    #[test]
    fn unknown_down_target_is_refused() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        assert!(matches!(
            database.migrate_down_to(99),
            Err(DbError::UnknownMigration(99))
        ));
    }

    // --- schema v4: tool-turn persistence ---------------------------------

    use super::ToolTurnRecord;
    use hiverouter_contract::event::ToolOutcome;
    use hiverouter_contract::{RunId, TaskId, Timestamp};

    fn insert_run(database: &Database, run_id: RunId) {
        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO runs(id, project_id, prompt, status, created_at, updated_at)
                     VALUES (?1, 'project-1', 'persist tool turns', 'queued', ?2, ?2)",
                    rusqlite::params![run_id.to_string(), Timestamp::now().as_str()],
                )?;
                Ok(())
            })
            .expect("insert run row");
    }

    fn sample_turn(run_id: RunId, task_id: TaskId) -> ToolTurnRecord {
        ToolTurnRecord {
            id: uuid::Uuid::now_v7().to_string(),
            run_id,
            task_id,
            tool_call_id: "call_01".to_owned(),
            tool: "file_read".to_owned(),
            args_sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
                .to_owned(),
            outcome: ToolOutcome::Refused,
            code: Some("E_TOOL_NOT_ALLOWED".to_owned()),
            bytes: Some(0),
            sha256: None,
            turn: 1,
            created_at: Timestamp::now(),
        }
    }

    #[test]
    fn tool_turn_roundtrip_and_run_counters() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);

        assert_eq!(
            database.run_tool_counters(run_id).expect("fresh counters"),
            (0, 0)
        );

        let record = sample_turn(run_id, task_id);
        database.record_tool_turn(&record).expect("record turn");
        database
            .increment_run_tool_counters(run_id, 1, 1)
            .expect("increment");
        database
            .increment_run_tool_counters(run_id, 0, 1)
            .expect("increment");
        assert_eq!(
            database.run_tool_counters(run_id).expect("counters"),
            (1, 2)
        );

        let turns = database.tool_turns_for_run(run_id).expect("read turns");
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0], record);

        let empty_run = RunId::new();
        insert_run(&database, empty_run);
        assert!(database
            .tool_turns_for_run(empty_run)
            .expect("no turns")
            .is_empty());
        assert_eq!(
            database.run_tool_counters(empty_run).expect("counters"),
            (0, 0)
        );

        let unknown = RunId::new();
        assert!(database.run_tool_counters(unknown).is_err());
        assert!(database
            .increment_run_tool_counters(unknown, 1, 0)
            .is_err());
    }

    #[test]
    fn task_turn_count_is_per_task_not_per_run() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        let first = TaskId::new();
        let second = TaskId::new();
        insert_run(&database, run_id);

        let mut turn_one = sample_turn(run_id, first);
        database.record_tool_turn(&turn_one).expect("first task turn 1");
        turn_one.id = uuid::Uuid::now_v7().to_string();
        turn_one.tool_call_id = "call_02".to_owned();
        turn_one.turn = 2;
        database.record_tool_turn(&turn_one).expect("first task turn 2");
        let mut other = sample_turn(run_id, second);
        other.id = uuid::Uuid::now_v7().to_string();
        other.tool_call_id = "call_03".to_owned();
        database.record_tool_turn(&other).expect("second task turn 1");

        assert_eq!(database.task_tool_turns(run_id, first).expect("first"), 2);
        assert_eq!(database.task_tool_turns(run_id, second).expect("second"), 1);
        assert_eq!(database.task_tool_turns(run_id, TaskId::new()).expect("unknown"), 0);
    }

    #[test]
    fn duplicate_tool_turn_is_refused_and_not_persisted() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);

        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("first record");
        let mut duplicate = sample_turn(run_id, task_id);
        duplicate.id = uuid::Uuid::now_v7().to_string();

        let error = database
            .record_tool_turn(&duplicate)
            .expect_err("same (run_id, tool_call_id) must be refused");
        assert!(
            matches!(error, DbError::Sqlite(_)),
            "duplicate is a SQLite constraint error, got {error:?}"
        );
        assert_eq!(
            database.tool_turns_for_run(run_id).expect("turns").len(),
            1,
            "the duplicate must not be persisted"
        );
    }

    #[test]
    fn tool_turn_migration_rolls_back_and_reapplies() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(database.migrate().expect("migrate"), 32);
        let run_id = RunId::new();
        let task_id = TaskId::new();
        insert_run(&database, run_id);
        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("record turn");
        database
            .increment_run_tool_counters(run_id, 1, 1)
            .expect("increment");
        assert_eq!(
            database.run_tool_counters(run_id).expect("counters"),
            (1, 1)
        );

        assert_eq!(database.migrate_down_to(3).expect("down v4"), 3);
        assert!(
            database.tool_turns_for_run(run_id).is_err(),
            "v3 has no tool_turns table"
        );
        assert!(
            database.run_tool_counters(run_id).is_err(),
            "v3 runs has no tool counter columns"
        );
        let tool_tables: i64 = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'tool_turns'",
                    [],
                    |row| row.get(0),
                )
            })
            .expect("table probe");
        assert_eq!(tool_tables, 0, "down migration must drop tool_turns");

        assert_eq!(database.migrate().expect("re-migrate"), 32);
        assert_eq!(
            database.run_tool_counters(run_id).expect("recreated counters"),
            (0, 0),
            "re-added counter columns start at their default"
        );
        database
            .record_tool_turn(&sample_turn(run_id, task_id))
            .expect("record after re-up");
        assert_eq!(
            database.tool_turns_for_run(run_id).expect("turns").len(),
            1
        );
    }

    #[test]
    fn schema_newer_than_binary_is_refused_without_running_migrations() {
        let temp = tempfile::tempdir().expect("temp");
        let path = temp.path().join("newer.db");
        let database = Database::open(&path).expect("open");
        database.migrate().expect("initial migrate");
        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO schema_migrations(version, name, applied_at) VALUES (33, 'future', ?1)",
                    rusqlite::params![Timestamp::now().as_str()],
                )
            })
            .expect("insert future version");
        let before = database.applied_migrations().expect("before");
        let error = database.migrate().expect_err("newer schema must refuse");
        assert!(matches!(
            error,
            DbError::SchemaNewer {
                current: 33,
                latest: 32
            }
        ));
        assert_eq!(database.applied_migrations().expect("after"), before);
        assert_eq!(database.schema_version().expect("version"), 33);
    }

    #[test]
    fn pending_migration_writes_a_verified_backup() {
        let temp = tempfile::tempdir().expect("temp");
        let path = temp.path().join("backup.db");
        let database = Database::open(&path).expect("open");
        database
            .with_connection(|connection| {
                connection.execute_batch(
                    "CREATE TABLE schema_migrations (
                        version    INTEGER PRIMARY KEY,
                        name       TEXT NOT NULL,
                        applied_at TEXT NOT NULL
                    );",
                )?;
                for migration in MIGRATIONS.iter().take(10) {
                    connection.execute_batch(migration.up)?;
                    connection.execute(
                        "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![
                            migration.version,
                            migration.name,
                            Timestamp::now().as_str()
                        ],
                    )?;
                }
                Ok(())
            })
            .expect("seed v10 schema");
        database.migrate().expect("upgrade to 11");
        let backups: Vec<PathBuf> = std::fs::read_dir(temp.path())
            .expect("dir")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|entry| {
                entry
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(".pre-v10-") && name.ends_with(".bak"))
            })
            .collect();
        assert_eq!(backups.len(), 1, "one pre-v10 backup must be written");
        let backup = Connection::open(&backups[0]).expect("open backup");
        let integrity: String = backup
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .expect("integrity");
        assert_eq!(integrity, "ok");
    }

    // --- schema v25: sessions, session_events, todos, run policy ----------

    fn table_names(database: &Database) -> Vec<String> {
        database
            .with_connection(|connection| {
                let mut statement = connection
                    .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
                let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
                let mut names = Vec::new();
                for row in rows {
                    names.push(row?);
                }
                Ok(names)
            })
            .expect("table names")
    }

    fn column_names(database: &Database, table: &str) -> Vec<String> {
        database
            .with_connection(|connection| {
                let mut statement = connection.prepare(&format!("PRAGMA table_info({table})"))?;
                let rows = statement.query_map([], |row| row.get::<_, String>(1))?;
                let mut names = Vec::new();
                for row in rows {
                    names.push(row?);
                }
                Ok(names)
            })
            .expect("column names")
    }

    /// Applies only the migrations up to `target`, producing a genuine
    /// `target`-era database without relying on the down migrations.
    fn seed_schema(database: &Database, target: i64) {
        database
            .with_connection(|connection| {
                connection.execute_batch(
                    "CREATE TABLE IF NOT EXISTS schema_migrations (
                        version    INTEGER PRIMARY KEY,
                        name       TEXT NOT NULL,
                        applied_at TEXT NOT NULL
                    );",
                )?;
                for migration in MIGRATIONS.iter().filter(|m| m.version <= target) {
                    connection.execute_batch(migration.up)?;
                    connection.execute(
                        "INSERT INTO schema_migrations(version, name, applied_at) VALUES (?1, ?2, ?3)",
                        rusqlite::params![
                            migration.version,
                            migration.name,
                            Timestamp::now().as_str()
                        ],
                    )?;
                }
                Ok(())
            })
            .expect("seed schema");
    }

    #[test]
    fn v25_up_adds_sessions_todos_and_run_policy_columns() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(
            database.migrate().expect("migrate"),
            MIGRATIONS.last().expect("migrations").version
        );
        let tables = table_names(&database);
        for expected in ["sessions", "session_events", "todos"] {
            assert!(
                tables.iter().any(|name| name == expected),
                "{expected} missing from {tables:?}"
            );
        }
        let columns = column_names(&database, "runs");
        assert!(
            columns.iter().any(|name| name == "session_id"),
            "runs.session_id missing from {columns:?}"
        );
        assert!(
            columns.iter().any(|name| name == "permission_policy"),
            "runs.permission_policy missing from {columns:?}"
        );
        let applied = database.applied_migrations().expect("migrations");
        let v25 = applied
            .iter()
            .find(|(version, _, _)| *version == 25)
            .expect("v25 must be applied");
        assert_eq!(v25.1, "sessions_policy_todos");
    }

    #[test]
    fn v25_defaults_runs_to_classic_and_rejects_unknown_policies() {
        let database = Database::open_in_memory().expect("in-memory db");
        database.migrate().expect("migrate");
        let run_id = RunId::new();
        insert_run(&database, run_id);
        let policy: String = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT permission_policy FROM runs WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .expect("stored policy");
        assert_eq!(policy, "classic", "a run must default to classic");
        let rejected = database.with_connection(|connection| {
            connection.execute(
                "UPDATE runs SET permission_policy = 'yolo' WHERE id = ?1",
                rusqlite::params![run_id.to_string()],
            )
        });
        assert!(
            rejected.is_err(),
            "the CHECK constraint must reject an unknown policy"
        );
        let session_rejected = database.with_connection(|connection| {
            connection.execute(
                "INSERT INTO sessions(id, project_id, name, workspace_root, permission_policy, created_at, updated_at)
                 VALUES ('s1', 'p', 'n', '/w', 'yolo', '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z')",
                [],
            )
        });
        assert!(
            session_rejected.is_err(),
            "the session CHECK constraint must reject an unknown policy"
        );
    }

    #[test]
    fn v24_database_upgrades_to_v25_with_runs_still_readable() {
        let database = Database::open_in_memory().expect("in-memory db");
        seed_schema(&database, 24);
        assert_eq!(database.schema_version().expect("version"), 24);
        let run_id = RunId::new();
        // A v24-era insert: the new columns do not exist yet.
        insert_run(&database, run_id);
        assert_eq!(
            database.migrate().expect("upgrade"),
            MIGRATIONS.last().expect("migrations").version
        );
        let row: (String, String, String, String) = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT project_id, prompt, status, permission_policy FROM runs WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                )
            })
            .expect("old run row after upgrade");
        assert_eq!(row.0, "project-1");
        assert_eq!(row.1, "persist tool turns");
        assert_eq!(row.2, "queued");
        assert_eq!(row.3, "classic", "old runs adopt the classic default");
        let session_id: Option<String> = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT session_id FROM runs WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .expect("old run session_id");
        assert_eq!(session_id, None);
    }

    #[test]
    fn v25_down_restores_v24_shape_and_reapplies() {
        let database = Database::open_in_memory().expect("in-memory db");
        assert_eq!(
            database.migrate().expect("migrate"),
            MIGRATIONS.last().expect("migrations").version
        );
        let run_id = RunId::new();
        insert_run(&database, run_id);
        database
            .with_connection(|connection| {
                connection.execute(
                    "INSERT INTO sessions(id, project_id, name, workspace_root, permission_policy, created_at, updated_at)
                     VALUES ('session-1', 'p', 's', '/w', 'trusted', ?1, ?1)",
                    rusqlite::params![Timestamp::now().as_str()],
                )?;
                connection.execute(
                    "INSERT INTO todos(id, session_id, seq, text, state, created_at)
                     VALUES ('todo-1', 'session-1', 1, 'one', 'open', ?1)",
                    rusqlite::params![Timestamp::now().as_str()],
                )?;
                connection.execute(
                    "INSERT INTO session_events(id, session_id, kind, detail_json, created_at)
                     VALUES ('event-1', 'session-1', 'created', '{}', ?1)",
                    rusqlite::params![Timestamp::now().as_str()],
                )?;
                connection.execute(
                    "UPDATE runs SET session_id = 'session-1', permission_policy = 'trusted' WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                )?;
                Ok(())
            })
            .expect("seed v25 rows");

        assert_eq!(database.migrate_down_to(24).expect("down v25"), 24);
        let tables = table_names(&database);
        for gone in ["sessions", "session_events", "todos"] {
            assert!(
                !tables.iter().any(|name| name == gone),
                "{gone} must be dropped by the v25 down migration, got {tables:?}"
            );
        }
        let columns = column_names(&database, "runs");
        assert!(
            !columns.iter().any(|name| name == "session_id"),
            "runs.session_id must be dropped, got {columns:?}"
        );
        assert!(
            !columns.iter().any(|name| name == "permission_policy"),
            "runs.permission_policy must be dropped, got {columns:?}"
        );
        // The pre-existing run survives the rollback.
        let prompt: String = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT prompt FROM runs WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .expect("run row after rollback");
        assert_eq!(prompt, "persist tool turns");

        // The forward migration re-applies cleanly and rebuilds the v25 surface.
        assert_eq!(
            database.migrate().expect("re-migrate"),
            MIGRATIONS.last().expect("migrations").version
        );
        let tables = table_names(&database);
        for expected in ["sessions", "session_events", "todos"] {
            assert!(tables.iter().any(|name| name == expected), "{expected} missing");
        }
        let policy: String = database
            .with_connection(|connection| {
                connection.query_row(
                    "SELECT permission_policy FROM runs WHERE id = ?1",
                    rusqlite::params![run_id.to_string()],
                    |row| row.get(0),
                )
            })
            .expect("re-added policy column");
        assert_eq!(policy, "classic");
    }
}
