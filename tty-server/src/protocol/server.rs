use anyhow::{Result, anyhow};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{RwLock, mpsc};
use std::sync::Arc;
use bytes::{BytesMut, BufMut};
use tracing::{info, error, debug};

use crate::session::{SessionManager, StreamEvent};
use super::messages::{ClientMessage, ServerMessage, SessionInfo, MessageInfo};

pub async fn run(
    bind_addr: String,
    session_manager: Arc<RwLock<SessionManager>>,
) -> Result<()> {
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Native protocol server listening on {}", bind_addr);

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                debug!("New connection from {}", addr);
                let session_manager = session_manager.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, session_manager).await {
                        error!("Connection error from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    session_manager: Arc<RwLock<SessionManager>>,
) -> Result<()> {
    let mut authenticated_user: Option<String> = None;
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        // Read frame length (4 bytes, big-endian)
        let mut len_bytes = [0u8; 4];
        match socket.read_exact(&mut len_bytes).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Client disconnected");
                break;
            }
            Err(e) => return Err(e.into()),
        }

        let frame_len = u32::from_be_bytes(len_bytes) as usize;
        if frame_len > 1_000_000 {
            return Err(anyhow!("Frame too large: {} bytes", frame_len));
        }

        // Read frame data
        buffer.clear();
        buffer.resize(frame_len, 0);
        socket.read_exact(&mut buffer).await?;

        // Parse message
        let client_msg: ClientMessage = serde_json::from_slice(&buffer)?;
        debug!("Received: {:?}", client_msg);

        // Handle message
        let response = match client_msg {
            ClientMessage::Authenticate { token } => {
                // TODO: Implement proper JWT validation
                // For now, just accept any token and use it as user_id
                authenticated_user = Some(token.clone());
                ServerMessage::Authenticated { user_id: token }
            }

            ClientMessage::CreateSession { provider, model } => {
                let user_id = authenticated_user.as_ref()
                    .ok_or_else(|| anyhow!("Not authenticated"))?;

                let mut sm = session_manager.write().await;
                match sm.create_session(user_id, &provider, model) {
                    Ok(session_id) => ServerMessage::SessionCreated { session_id },
                    Err(e) => ServerMessage::Error {
                        code: "CREATE_SESSION_FAILED".to_string(),
                        message: e.to_string(),
                    },
                }
            }

            ClientMessage::SendMessage { session_id, content } => {
                if authenticated_user.is_none() {
                    ServerMessage::Error {
                        code: "NOT_AUTHENTICATED".to_string(),
                        message: "Not authenticated".to_string(),
                    }
                } else {
                    // Create channel for streaming response
                    let (tx, mut rx) = mpsc::unbounded_channel();

                    // Send message to provider
                    let sm = session_manager.read().await;
                    if let Err(e) = sm.send_message(&session_id, content, tx) {
                        ServerMessage::Error {
                            code: "SEND_MESSAGE_FAILED".to_string(),
                            message: e.to_string(),
                        }
                    } else {
                        // Stream tokens back to client
                        drop(sm);
                        while let Some(event) = rx.recv().await {
                            let msg = match &event {
                                StreamEvent::Token(text) => ServerMessage::Token {
                                    session_id: session_id.clone(),
                                    content: text.clone(),
                                },
                                StreamEvent::Complete => ServerMessage::MessageComplete {
                                    session_id: session_id.clone(),
                                },
                                StreamEvent::Error(err) => ServerMessage::Error {
                                    code: "STREAM_ERROR".to_string(),
                                    message: err.clone(),
                                },
                            };

                            send_message(&mut socket, &msg).await?;

                            if matches!(event, StreamEvent::Complete | StreamEvent::Error(_)) {
                                break;
                            }
                        }
                        continue; // Skip sending response at the end
                    }
                }
            }

            ClientMessage::ListSessions => {
                let user_id = authenticated_user.as_ref()
                    .ok_or_else(|| anyhow!("Not authenticated"))?;

                let sm = session_manager.read().await;
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
                        ServerMessage::SessionsList { sessions }
                    }
                    Err(e) => ServerMessage::Error {
                        code: "LIST_SESSIONS_FAILED".to_string(),
                        message: e.to_string(),
                    },
                }
            }

            ClientMessage::LoadSession { session_id } => {
                if authenticated_user.is_none() {
                    ServerMessage::Error {
                        code: "NOT_AUTHENTICATED".to_string(),
                        message: "Not authenticated".to_string(),
                    }
                } else {
                    let sm = session_manager.read().await;
                    match sm.get_session(&session_id) {
                        Ok(Some(session)) => {
                            match sm.get_messages(&session_id) {
                                Ok(messages) => {
                                    let session_info = SessionInfo {
                                        id: session.id,
                                        provider: session.provider,
                                        model: session.model,
                                        created_at: session.created_at,
                                        updated_at: session.updated_at,
                                    };
                                    let message_infos: Vec<MessageInfo> = messages.into_iter()
                                        .map(|m| MessageInfo {
                                            id: m.id,
                                            role: m.role,
                                            content: m.content,
                                            created_at: m.created_at,
                                        })
                                        .collect();
                                    ServerMessage::SessionLoaded {
                                        session: session_info,
                                        messages: message_infos,
                                    }
                                }
                                Err(e) => ServerMessage::Error {
                                    code: "LOAD_MESSAGES_FAILED".to_string(),
                                    message: e.to_string(),
                                },
                            }
                        }
                        Ok(None) => ServerMessage::Error {
                            code: "SESSION_NOT_FOUND".to_string(),
                            message: "Session not found".to_string(),
                        },
                        Err(e) => ServerMessage::Error {
                            code: "LOAD_SESSION_FAILED".to_string(),
                            message: e.to_string(),
                        },
                    }
                }
            }

            ClientMessage::DeleteSession { session_id } => {
                if authenticated_user.is_none() {
                    ServerMessage::Error {
                        code: "NOT_AUTHENTICATED".to_string(),
                        message: "Not authenticated".to_string(),
                    }
                } else {
                    let mut sm = session_manager.write().await;
                    match sm.delete_session(&session_id) {
                        Ok(_) => ServerMessage::MessageComplete { session_id },
                        Err(e) => ServerMessage::Error {
                            code: "DELETE_SESSION_FAILED".to_string(),
                            message: e.to_string(),
                        },
                    }
                }
            }
        };

        send_message(&mut socket, &response).await?;
    }

    Ok(())
}

async fn send_message(socket: &mut TcpStream, msg: &ServerMessage) -> Result<()> {
    let json = serde_json::to_vec(msg)?;
    let len = json.len() as u32;

    let mut frame = BytesMut::with_capacity(4 + json.len());
    frame.put_u32(len);
    frame.put_slice(&json);

    socket.write_all(&frame).await?;
    Ok(())
}
