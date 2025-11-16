use anyhow::{Result, Context};
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;

pub mod models;
use models::{Session, Message, User};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)
            .context(format!("Failed to open database at {}", path))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn migrate(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(user_id) REFERENCES users(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session
             ON messages(session_id, created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_user
             ON sessions(user_id, updated_at DESC)",
            [],
        )?;

        Ok(())
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> Result<User> {
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            created_at: Utc::now().timestamp(),
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![user.id, user.username, user.password_hash, user.created_at],
        )?;

        Ok(user)
    }

    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, created_at FROM users WHERE username = ?1"
        )?;

        let user = stmt.query_row(params![username], |row| {
            Ok(User {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                created_at: row.get(3)?,
            })
        }).optional()?;

        Ok(user)
    }

    pub fn create_session(&self, user_id: &str, provider: &str, model: &str) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, user_id, provider, model, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id,
                session.user_id,
                session.provider,
                session.model,
                session.created_at,
                session.updated_at
            ],
        )?;

        Ok(session)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, provider, model, created_at, updated_at
             FROM sessions WHERE id = ?1"
        )?;

        let session = stmt.query_row(params![session_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                user_id: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).optional()?;

        Ok(session)
    }

    pub fn list_sessions(&self, user_id: &str) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, provider, model, created_at, updated_at
             FROM sessions WHERE user_id = ?1 ORDER BY updated_at DESC"
        )?;

        let sessions = stmt.query_map(params![user_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                user_id: row.get(1)?,
                provider: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<Message> {
        let message = Message {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            created_at: Utc::now().timestamp(),
        };

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                message.id,
                message.session_id,
                message.role,
                message.content,
                message.created_at
            ],
        )?;

        // Update session updated_at
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![Utc::now().timestamp(), session_id],
        )?;

        Ok(message)
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, created_at
             FROM messages WHERE session_id = ?1 ORDER BY created_at ASC"
        )?;

        let messages = stmt.query_map(params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(messages)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Delete messages first
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![session_id])?;

        // Delete session
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;

        Ok(())
    }
}
