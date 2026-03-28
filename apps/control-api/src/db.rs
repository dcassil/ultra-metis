//! SQLite database initialization and schema management.

use rusqlite::{Connection, Result};

/// Initialize the database with all required tables and seed data.
pub fn init_db(conn: &Connection) -> Result<()> {
    create_tables(conn)?;
    seed_defaults(conn)?;
    Ok(())
}

fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS orgs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL REFERENCES orgs(id),
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL REFERENCES orgs(id),
            team_id TEXT NOT NULL REFERENCES teams(id),
            email TEXT,
            display_name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS roles (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS user_roles (
            user_id TEXT NOT NULL REFERENCES users(id),
            role_id TEXT NOT NULL REFERENCES roles(id),
            PRIMARY KEY (user_id, role_id)
        );

        CREATE TABLE IF NOT EXISTS machines (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            name TEXT NOT NULL,
            platform TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            trust_tier TEXT NOT NULL DEFAULT 'untrusted',
            capabilities TEXT,
            last_heartbeat TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS machine_repos (
            id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL REFERENCES machines(id),
            repo_path TEXT NOT NULL,
            repo_name TEXT,
            last_seen TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS machine_tokens (
            token TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            description TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at TEXT
        );

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            machine_id TEXT NOT NULL REFERENCES machines(id),
            repo_path TEXT NOT NULL,
            title TEXT NOT NULL,
            instructions TEXT NOT NULL,
            autonomy_level TEXT NOT NULL DEFAULT 'normal',
            work_item_id TEXT,
            context TEXT,
            state TEXT NOT NULL DEFAULT 'starting',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            started_at TEXT,
            completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS session_events (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            from_state TEXT,
            to_state TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            metadata TEXT
        );

        CREATE TABLE IF NOT EXISTS session_commands (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            machine_id TEXT NOT NULL REFERENCES machines(id),
            command_type TEXT NOT NULL,
            payload TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            delivered_at TEXT
        );

        CREATE TABLE IF NOT EXISTS machine_policies (
            id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL UNIQUE REFERENCES machines(id),
            allowed_categories TEXT NOT NULL DEFAULT '[]',
            blocked_categories TEXT NOT NULL DEFAULT '[]',
            max_autonomy_level TEXT NOT NULL DEFAULT 'autonomous',
            session_mode TEXT NOT NULL DEFAULT 'normal',
            require_approval_for TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS repo_policies (
            id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL REFERENCES machines(id),
            repo_path TEXT NOT NULL,
            allowed_categories TEXT NOT NULL DEFAULT '[]',
            blocked_categories TEXT NOT NULL DEFAULT '[]',
            max_autonomy_level TEXT,
            require_approval_for TEXT NOT NULL DEFAULT '[]',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(machine_id, repo_path)
        );

        CREATE TABLE IF NOT EXISTS policy_violations (
            id TEXT PRIMARY KEY,
            session_id TEXT,
            machine_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            action TEXT NOT NULL,
            policy_scope TEXT NOT NULL,
            reason TEXT NOT NULL,
            repo_path TEXT,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );
        ",
    )?;
    Ok(())
}

fn seed_defaults(conn: &Connection) -> Result<()> {
    seed_org_and_team(conn)?;
    seed_user(conn)?;
    seed_roles(conn)?;
    seed_token(conn)?;
    Ok(())
}

fn seed_org_and_team(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO orgs (id, name) VALUES (?1, ?2)",
        ("org-default", "Default Organization"),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO teams (id, org_id, name) VALUES (?1, ?2, ?3)",
        ("team-default", "org-default", "Default Team"),
    )?;
    Ok(())
}

fn seed_user(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO users (id, org_id, team_id, display_name) VALUES (?1, ?2, ?3, ?4)",
        ("user-default", "org-default", "team-default", "Default User"),
    )?;
    Ok(())
}

fn seed_roles(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO roles (id, name, description) VALUES (?1, ?2, ?3)",
        ("role-admin", "admin", "Full access"),
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?1, ?2)",
        ("user-default", "role-admin"),
    )?;
    Ok(())
}

fn seed_token(conn: &Connection) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO machine_tokens (token, user_id, description) VALUES (?1, ?2, ?3)",
        (
            "cadre-mvp-static-token",
            "user-default",
            "MVP static token for development",
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        // Verify tables exist by querying them
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM orgs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM machine_tokens", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_init_db_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        init_db(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_seed_data_correct() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();

        let token_user: String = conn
            .query_row(
                "SELECT user_id FROM machine_tokens WHERE token = ?1",
                ["cadre-mvp-static-token"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(token_user, "user-default");
    }
}
