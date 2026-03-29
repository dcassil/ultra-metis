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

        CREATE TABLE IF NOT EXISTS session_output_events (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            event_type TEXT NOT NULL,
            category TEXT,
            content TEXT NOT NULL DEFAULT '',
            metadata TEXT,
            sequence_num INTEGER NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS pending_approvals (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id),
            question TEXT NOT NULL,
            options TEXT NOT NULL DEFAULT '[]',
            context TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            response_choice TEXT,
            response_note TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            responded_at TEXT
        );

        CREATE TABLE IF NOT EXISTS session_outcomes (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL UNIQUE REFERENCES sessions(id),
            status TEXT NOT NULL,
            summary TEXT NOT NULL DEFAULT '',
            artifacts TEXT NOT NULL DEFAULT '[]',
            next_steps TEXT NOT NULL DEFAULT '',
            event_count INTEGER NOT NULL DEFAULT 0,
            intervention_count INTEGER NOT NULL DEFAULT 0,
            duration_seconds INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            session_id TEXT REFERENCES sessions(id),
            notification_type TEXT NOT NULL,
            priority TEXT NOT NULL DEFAULT 'normal',
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            deep_link TEXT,
            read_at TEXT,
            dismissed_at TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS device_tokens (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            token TEXT NOT NULL UNIQUE,
            platform TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS notification_preferences (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL REFERENCES users(id),
            notification_type TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, notification_type)
        );

        CREATE TABLE IF NOT EXISTS machine_logs (
            id TEXT PRIMARY KEY,
            machine_id TEXT NOT NULL REFERENCES machines(id),
            timestamp TEXT NOT NULL DEFAULT (datetime('now')),
            level TEXT NOT NULL,
            target TEXT NOT NULL DEFAULT '',
            message TEXT NOT NULL DEFAULT '',
            fields_json TEXT
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

/// Update an existing machine's registration data without changing status or trust_tier.
///
/// Deletes existing `machine_repos` for the machine and re-inserts from the request.
pub fn update_machine_registration(
    conn: &Connection,
    machine_id: &str,
    name: &str,
    platform: &str,
    capabilities: Option<&str>,
    repos: Option<&[crate::models::RepoInfo]>,
    now: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE machines SET name = ?1, platform = ?2, capabilities = ?3, last_heartbeat = ?4, updated_at = ?5 WHERE id = ?6",
        rusqlite::params![name, platform, capabilities, now, now, machine_id],
    )?;

    // Replace machine_repos: delete old, insert new
    conn.execute(
        "DELETE FROM machine_repos WHERE machine_id = ?1",
        [machine_id],
    )?;

    if let Some(repos) = repos {
        for repo in repos {
            let repo_id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO machine_repos (id, machine_id, repo_path, repo_name, last_seen)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![repo_id, machine_id, repo.path, repo.name, now],
            )?;
        }
    }

    Ok(())
}

/// Error returned when a machine cannot be deleted due to active sessions.
#[derive(Debug)]
pub enum DeleteMachineError {
    ActiveSessions(i64),
    NotFound,
    Db(rusqlite::Error),
}

impl std::fmt::Display for DeleteMachineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ActiveSessions(n) => write!(f, "machine has {n} active session(s)"),
            Self::NotFound => write!(f, "machine not found"),
            Self::Db(e) => write!(f, "database error: {e}"),
        }
    }
}

impl From<rusqlite::Error> for DeleteMachineError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

/// Delete a single machine and all its related data.
///
/// Returns an error if the machine has active sessions (not completed/failed/stopped).
pub fn delete_machine(conn: &Connection, machine_id: &str) -> std::result::Result<(), DeleteMachineError> {
    // Check that the machine exists
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM machines WHERE id = ?1)",
            [machine_id],
            |row| row.get(0),
        )
        .map_err(DeleteMachineError::Db)?;

    if !exists {
        return Err(DeleteMachineError::NotFound);
    }

    // Check for active sessions
    let active_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM sessions WHERE machine_id = ?1 AND state NOT IN ('completed', 'failed', 'stopped')",
            [machine_id],
            |row| row.get(0),
        )
        .map_err(DeleteMachineError::Db)?;

    if active_count > 0 {
        return Err(DeleteMachineError::ActiveSessions(active_count));
    }

    // Delete from related tables in dependency order
    conn.execute("DELETE FROM machine_logs WHERE machine_id = ?1", [machine_id])?;
    conn.execute(
        "DELETE FROM pending_approvals WHERE session_id IN (SELECT id FROM sessions WHERE machine_id = ?1)",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM session_outcomes WHERE session_id IN (SELECT id FROM sessions WHERE machine_id = ?1)",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM session_output_events WHERE session_id IN (SELECT id FROM sessions WHERE machine_id = ?1)",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM session_events WHERE session_id IN (SELECT id FROM sessions WHERE machine_id = ?1)",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM session_commands WHERE machine_id = ?1",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM notifications WHERE session_id IN (SELECT id FROM sessions WHERE machine_id = ?1)",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM policy_violations WHERE machine_id = ?1",
        [machine_id],
    )?;
    conn.execute(
        "DELETE FROM sessions WHERE machine_id = ?1",
        [machine_id],
    )?;
    conn.execute("DELETE FROM repo_policies WHERE machine_id = ?1", [machine_id])?;
    conn.execute("DELETE FROM machine_policies WHERE machine_id = ?1", [machine_id])?;
    conn.execute("DELETE FROM machine_repos WHERE machine_id = ?1", [machine_id])?;
    conn.execute("DELETE FROM machines WHERE id = ?1", [machine_id])?;

    Ok(())
}

/// Delete all offline machines that have no active sessions.
///
/// Returns the number of machines deleted.
pub fn delete_offline_machines(conn: &Connection) -> std::result::Result<i64, DeleteMachineError> {
    // Find offline machines: last_heartbeat is either NULL or older than 5 minutes,
    // matching the connectivity_status logic in the models.
    let mut stmt = conn.prepare(
        "SELECT id FROM machines
         WHERE (last_heartbeat IS NULL
                OR last_heartbeat < datetime('now', '-5 minutes'))
           AND id NOT IN (
               SELECT DISTINCT machine_id FROM sessions
               WHERE state NOT IN ('completed', 'failed', 'stopped')
           )",
    )?;

    let ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<Vec<String>, _>>()?;

    let mut count: i64 = 0;
    for id in &ids {
        match delete_machine(conn, id) {
            Ok(()) => count += 1,
            Err(DeleteMachineError::ActiveSessions(_)) => {
                // Skip — race condition, session appeared between our check and delete
            }
            Err(e) => return Err(e),
        }
    }

    Ok(count)
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
