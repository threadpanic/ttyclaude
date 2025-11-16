use axum::{
    extract::{State, Path, WebSocketUpgrade, ws::{WebSocket, Message}},
    response::IntoResponse,
    Json, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, error};

use super::server::AppState;
use crate::protocol::messages::{SessionInfo, MessageInfo};
use crate::session::StreamEvent;

// Request/Response types
#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionInfo>,
}

#[derive(Debug, Serialize)]
pub struct GetSessionResponse {
    pub session: SessionInfo,
}

#[derive(Debug, Serialize)]
pub struct GetMessagesResponse {
    pub messages: Vec<MessageInfo>,
}

// Handlers
pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<ListSessionsResponse>, StatusCode> {
    // TODO: Extract user_id from JWT
    let user_id = "default_user";

    let sm = state.read().await;
    match sm.list_sessions(user_id) {
        Ok(sessions) => {
            let sessions: Vec<SessionInfo> = sessions.into_iter()
                .map(|s| SessionInfo {
                    id: s.id,
                    provider: s.provider,
                    model: s.model,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                })
                .collect();
            Ok(Json(ListSessionsResponse { sessions }))
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, StatusCode> {
    // TODO: Extract user_id from JWT
    let user_id = "default_user";

    let mut sm = state.write().await;
    match sm.create_session(user_id, &req.provider, req.model) {
        Ok(session_id) => Ok(Json(CreateSessionResponse { session_id })),
        Err(e) => {
            error!("Failed to create session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetSessionResponse>, StatusCode> {
    let sm = state.read().await;
    match sm.get_session(&id) {
        Ok(Some(session)) => {
            let session_info = SessionInfo {
                id: session.id,
                provider: session.provider,
                model: session.model,
                created_at: session.created_at,
                updated_at: session.updated_at,
            };
            Ok(Json(GetSessionResponse { session: session_info }))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut sm = state.write().await;
    match sm.delete_session(&id) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("Failed to delete session: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<GetMessagesResponse>, StatusCode> {
    let sm = state.read().await;
    match sm.get_messages(&id) {
        Ok(messages) => {
            let message_infos: Vec<MessageInfo> = messages.into_iter()
                .map(|m| MessageInfo {
                    id: m.id,
                    role: m.role,
                    content: m.content,
                    created_at: m.created_at,
                })
                .collect();
            Ok(Json(GetMessagesResponse { messages: message_infos }))
        }
        Err(e) => {
            error!("Failed to get messages: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// WebSocket handler
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsClientMessage {
    #[serde(rename = "authenticate")]
    Authenticate { token: String },

    #[serde(rename = "send_message")]
    SendMessage {
        session_id: String,
        content: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum WsServerMessage {
    #[serde(rename = "authenticated")]
    Authenticated { user_id: String },

    #[serde(rename = "token")]
    Token {
        session_id: String,
        content: String,
    },

    #[serde(rename = "message_complete")]
    MessageComplete { session_id: String },

    #[serde(rename = "error")]
    Error { message: String },
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    debug!("WebSocket connection established");
    let mut _authenticated_user: Option<String> = None;

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<WsClientMessage>(&text) {
                    Ok(client_msg) => {
                        match client_msg {
                            WsClientMessage::Authenticate { token } => {
                                // TODO: Validate JWT
                                _authenticated_user = Some(token.clone());
                                let response = WsServerMessage::Authenticated { user_id: token };
                                if let Ok(json) = serde_json::to_string(&response) {
                                    let _ = socket.send(Message::Text(json)).await;
                                }
                            }
                            WsClientMessage::SendMessage { session_id, content } => {
                                let (tx, mut rx) = mpsc::unbounded_channel();

                                let sm = state.read().await;
                                if let Err(e) = sm.send_message(&session_id, content, tx) {
                                    let response = WsServerMessage::Error {
                                        message: e.to_string(),
                                    };
                                    if let Ok(json) = serde_json::to_string(&response) {
                                        let _ = socket.send(Message::Text(json)).await;
                                    }
                                    continue;
                                }
                                drop(sm);

                                // Stream tokens
                                while let Some(event) = rx.recv().await {
                                    let response = match &event {
                                        StreamEvent::Token(text) => WsServerMessage::Token {
                                            session_id: session_id.clone(),
                                            content: text.clone(),
                                        },
                                        StreamEvent::Complete => WsServerMessage::MessageComplete {
                                            session_id: session_id.clone(),
                                        },
                                        StreamEvent::Error(err) => WsServerMessage::Error {
                                            message: err.clone(),
                                        },
                                    };

                                    if let Ok(json) = serde_json::to_string(&response) {
                                        if socket.send(Message::Text(json)).await.is_err() {
                                            break;
                                        }
                                    }

                                    if matches!(event, StreamEvent::Complete | StreamEvent::Error(_)) {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse WebSocket message: {}", e);
                        let response = WsServerMessage::Error {
                            message: format!("Invalid message: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&response) {
                            let _ = socket.send(Message::Text(json)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                debug!("WebSocket connection closed");
                break;
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }
}
