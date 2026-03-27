use anyhow::{Context, Result};
use rusqlite::Connection;

/// SQL statements to create the schema. Each string is executed in order.
const SCHEMA_MIGRATIONS: &[&str] = &[
    // -- Multi-tenancy scaffolding --
    "CREATE TABLE IF NOT EXISTS orgs (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE IF NOT EXISTS teams (
        id TEXT PRIMARY KEY,
        org_id TEXT NOT NULL REFERENCES orgs(id),
        name TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE IF NOT EXISTS users (
        id TEXT PRIMARY KEY,
        org_id TEXT NOT NULL REFERENCES orgs(id),
        team_id TEXT NOT NULL REFERENCES teams(id),
        name TEXT NOT NULL,
        email TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE TABLE IF NOT EXISTS roles (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        permissions_json TEXT NOT NULL DEFAULT '{}'
    );",
    "CREATE TABLE IF NOT EXISTS user_roles (
        user_id TEXT NOT NULL REFERENCES users(id),
        role_id TEXT NOT NULL REFERENCES roles(id),
        PRIMARY KEY (user_id, role_id)
    );",
    // -- Machine registry --
    "CREATE TABLE IF NOT EXISTS machines (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        platform TEXT,
        status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'trusted', 'revoked')),
        trust_tier TEXT NOT NULL DEFAULT 'trusted' CHECK(trust_tier IN ('trusted', 'restricted')),
        last_heartbeat TEXT,
        user_id TEXT NOT NULL REFERENCES users(id),
        team_id TEXT NOT NULL REFERENCES teams(id),
        org_id TEXT NOT NULL REFERENCES orgs(id),
        metadata TEXT DEFAULT '{}',
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE INDEX IF NOT EXISTS idx_machines_user_id ON machines(user_id);",
    "CREATE TABLE IF NOT EXISTS machine_repos (
        id TEXT PRIMARY KEY,
        machine_id TEXT NOT NULL REFERENCES machines(id) ON DELETE CASCADE,
        repo_name TEXT NOT NULL,
        repo_path TEXT NOT NULL,
        cadre_managed INTEGER NOT NULL DEFAULT 0,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE INDEX IF NOT EXISTS idx_machine_repos_machine_id ON machine_repos(machine_id);",
    // -- Auth tokens --
    "CREATE TABLE IF NOT EXISTS machine_tokens (
        id TEXT PRIMARY KEY,
        token TEXT NOT NULL UNIQUE,
        user_id TEXT NOT NULL REFERENCES users(id),
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );",
    "CREATE INDEX IF NOT EXISTS idx_machine_tokens_token ON machine_tokens(token);",
];

/// Default seed data inserted on first run.
const SEED_STATEMENTS: &[&str] = &[
    "INSERT OR IGNORE INTO orgs (id, name) VALUES ('org-default', 'Default Organization');",
    "INSERT OR IGNORE INTO teams (id, org_id, name) VALUES ('team-default', 'org-default', 'Default Team');",
    "INSERT OR IGNORE INTO users (id, org_id, team_id, name, email) VALUES ('user-default', 'org-default', 'team-default', 'Default User', 'admin@localhost');",
    "INSERT OR IGNORE INTO roles (id, name, permissions_json) VALUES ('role-default', 'default', '{}');",
    "INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES ('user-default', 'role-default');",
    "INSERT OR IGNORE INTO machine_tokens (id, token, user_id) VALUES ('token-default', 'cadre-mvp-static-token', 'user-default');",
];

/// Open (or create) a SQLite database at `path`, run all migrations, and seed
/// default data.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or any migration fails.
pub fn init_db(path: &str) -> Result<Connection> {
    let conn = Connection::open(path).context("failed to open database")?;

    // Enable WAL mode and foreign keys
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .context("failed to set pragmas")?;

    run_migrations(&conn)?;
    seed_defaults(&conn)?;

    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    for (i, sql) in SCHEMA_MIGRATIONS.iter().enumerate() {
        conn.execute_batch(sql)
            .with_context(|| format!("migration {i} failed"))?;
    }
    Ok(())
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    for (i, sql) in SEED_STATEMENTS.iter().enumerate() {
        conn.execute_batch(sql)
            .with_context(|| format!("seed statement {i} failed"))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn in_memory_db() -> Connection {
        init_db(":memory:").expect("failed to init in-memory db")
    }

    #[test]
    fn init_creates_all_tables() {
        let conn = in_memory_db();

        let expected_tables = [
            "orgs",
            "teams",
            "users",
            "roles",
            "user_roles",
            "machines",
            "machine_repos",
            "machine_tokens",
        ];

        for table in &expected_tables {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            assert!(exists, "table '{table}' should exist");
        }
    }

    #[test]
    fn init_seeds_default_org() {
        let conn = in_memory_db();
        let name: String = conn
            .query_row(
                "SELECT name FROM orgs WHERE id = 'org-default'",
                [],
                |row| row.get(0),
            )
            .expect("default org should exist");
        assert_eq!(name, "Default Organization");
    }

    #[test]
    fn init_seeds_default_team() {
        let conn = in_memory_db();
        let name: String = conn
            .query_row(
                "SELECT name FROM teams WHERE id = 'team-default'",
                [],
                |row| row.get(0),
            )
            .expect("default team should exist");
        assert_eq!(name, "Default Team");
    }

    #[test]
    fn init_seeds_default_user() {
        let conn = in_memory_db();
        let (name, email): (String, String) = conn
            .query_row(
                "SELECT name, email FROM users WHERE id = 'user-default'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("default user should exist");
        assert_eq!(name, "Default User");
        assert_eq!(email, "admin@localhost");
    }

    #[test]
    fn init_seeds_default_role_and_assignment() {
        let conn = in_memory_db();
        let role_name: String = conn
            .query_row(
                "SELECT name FROM roles WHERE id = 'role-default'",
                [],
                |row| row.get(0),
            )
            .expect("default role should exist");
        assert_eq!(role_name, "default");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM user_roles WHERE user_id = 'user-default' AND role_id = 'role-default'",
                [],
                |row| row.get(0),
            )
            .expect("user_roles query should succeed");
        assert_eq!(count, 1, "default user should have the default role");
    }

    #[test]
    fn init_seeds_default_token() {
        let conn = in_memory_db();
        let token: String = conn
            .query_row(
                "SELECT token FROM machine_tokens WHERE id = 'token-default'",
                [],
                |row| row.get(0),
            )
            .expect("default token should exist");
        assert_eq!(token, "cadre-mvp-static-token");
    }

    #[test]
    fn init_is_idempotent() {
        // Running init twice on the same database should not fail.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.db");
        let path_str = path.to_str().unwrap();

        let _conn1 = init_db(path_str).expect("first init should succeed");
        drop(_conn1);

        let conn2 = init_db(path_str).expect("second init should succeed");

        // Seed data should still be present exactly once.
        let count: i64 = conn2
            .query_row("SELECT COUNT(*) FROM orgs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "should have exactly one org after double-init");
    }

    #[test]
    fn machines_table_enforces_status_check() {
        let conn = in_memory_db();

        // Insert a valid machine first
        conn.execute(
            "INSERT INTO machines (id, name, status, trust_tier, user_id, team_id, org_id)
             VALUES ('m-1', 'test', 'trusted', 'trusted', 'user-default', 'team-default', 'org-default')",
            [],
        )
        .expect("valid machine insert should succeed");

        // Attempt an invalid status
        let result = conn.execute(
            "INSERT INTO machines (id, name, status, trust_tier, user_id, team_id, org_id)
             VALUES ('m-bad', 'bad', 'invalid_status', 'trusted', 'user-default', 'team-default', 'org-default')",
            [],
        );
        assert!(result.is_err(), "invalid status should be rejected by CHECK constraint");
    }

    #[test]
    fn machines_table_enforces_trust_tier_check() {
        let conn = in_memory_db();

        let result = conn.execute(
            "INSERT INTO machines (id, name, status, trust_tier, user_id, team_id, org_id)
             VALUES ('m-bad', 'bad', 'pending', 'invalid_tier', 'user-default', 'team-default', 'org-default')",
            [],
        );
        assert!(
            result.is_err(),
            "invalid trust_tier should be rejected by CHECK constraint"
        );
    }

    #[test]
    fn indexes_exist() {
        let conn = in_memory_db();

        let expected_indexes = [
            "idx_machines_user_id",
            "idx_machine_repos_machine_id",
            "idx_machine_tokens_token",
        ];

        for idx in &expected_indexes {
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='index' AND name=?1",
                    [idx],
                    |row| row.get(0),
                )
                .unwrap_or(false);
            assert!(exists, "index '{idx}' should exist");
        }
    }
}
