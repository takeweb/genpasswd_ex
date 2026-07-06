use rusqlite::{Connection, Result, params};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

pub struct PasswordEntry {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub created_at: String,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS password_history (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                service    TEXT NOT NULL,
                username   TEXT NOT NULL DEFAULT '',
                password   TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_service
                ON password_history(service);",
        )?;

        // 既存DBにusernameカラムがなければ追加
        let has_username: bool = conn
            .prepare("PRAGMA table_info(password_history)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .any(|name| name.unwrap_or_default() == "username");
        if !has_username {
            conn.execute_batch(
                "ALTER TABLE password_history ADD COLUMN username TEXT NOT NULL DEFAULT '';",
            )?;
        }

        Ok(Self { conn })
    }

    pub fn save(&self, service: &str, username: &str, password: &str) -> Result<()> {
        let now = chrono::Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO password_history (service, username, password, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![service, username, password, now],
        )?;
        Ok(())
    }

    pub fn get_history(&self, service: &str) -> Result<Vec<PasswordEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, username, password, created_at
             FROM password_history
             WHERE service = ?1
             ORDER BY created_at DESC",
        )?;
        let entries = stmt
            .query_map(params![service], |row| {
                Ok(PasswordEntry {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    password: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>>>()?;
        Ok(entries)
    }

    pub fn list_services(&self) -> Result<Vec<(String, usize)>> {
        let mut stmt = self.conn.prepare(
            "SELECT service, COUNT(*) as cnt
             FROM password_history
             GROUP BY service
             ORDER BY service",
        )?;
        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?)))?
            .collect::<Result<Vec<_>>>()?;
        Ok(rows)
    }

    pub fn delete_service(&self, service: &str) -> Result<usize> {
        let n = self.conn.execute(
            "DELETE FROM password_history WHERE service = ?1",
            params![service],
        )?;
        Ok(n)
    }
}
