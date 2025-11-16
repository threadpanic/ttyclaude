use serde::{Deserialize, Serialize};

// Client → Server messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "authenticate")]
    Authenticate { token: String },

    #[serde(rename = "create_session")]
    CreateSession {
        provider: String,
        model: Option<String>,
    },

    #[serde(rename = "send_message")]
    SendMessage {
        session_id: String,
        content: String,
    },

    #[serde(rename = "list_sessions")]
    ListSessions,

    #[serde(rename = "load_session")]
    LoadSession { session_id: String },

    #[serde(rename = "delete_session")]
    DeleteSession { session_id: String },
}

// Server → Client messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "authenticated")]
    Authenticated { user_id: String },

    #[serde(rename = "session_created")]
    SessionCreated { session_id: String },

    #[serde(rename = "token")]
    Token {
        session_id: String,
        content: String,
    },

    #[serde(rename = "message_complete")]
    MessageComplete { session_id: String },

    #[serde(rename = "error")]
    Error { code: String, message: String },

    #[serde(rename = "sessions_list")]
    SessionsList {
        sessions: Vec<SessionInfo>,
    },

    #[serde(rename = "session_loaded")]
    SessionLoaded {
        session: SessionInfo,
        messages: Vec<MessageInfo>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: i64,
}
