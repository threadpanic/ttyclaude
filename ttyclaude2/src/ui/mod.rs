use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;

use crate::config::Config;
use crate::protocol::{Client, ClientMessage, ServerMessage};

pub struct App {
    client: Client,
    config: Config,
    mode: Mode,
    input_buffer: String,
    messages: VecDeque<DisplayMessage>,
    current_session: Option<String>,
    sessions: Vec<crate::protocol::SessionInfo>,
    status_message: String,
    authenticated: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Insert,
    SessionList,
}

#[derive(Debug, Clone)]
struct DisplayMessage {
    role: String,
    content: String,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let mut client = Client::connect(&config.server.address).await?;

        // Authenticate
        client.send(&ClientMessage::Authenticate {
            token: config.auth.token.clone(),
        }).await?;

        Ok(Self {
            client,
            config,
            mode: Mode::Normal,
            input_buffer: String::new(),
            messages: VecDeque::new(),
            current_session: None,
            sessions: Vec::new(),
            status_message: "Connected. Press 'i' to insert, 'n' for new session, 'l' to list sessions".to_string(),
            authenticated: false,
        })
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;

        match self.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('i') => {
                    self.mode = Mode::Insert;
                    self.status_message = "-- INSERT --".to_string();
                }
                KeyCode::Char('n') => {
                    self.create_session().await?;
                }
                KeyCode::Char('l') => {
                    self.list_sessions().await?;
                }
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.status_message = "Normal mode".to_string();
                }
                _ => {}
            },
            Mode::Insert => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.status_message = "Normal mode".to_string();
                }
                KeyCode::Enter => {
                    if !self.input_buffer.is_empty() {
                        self.send_message().await?;
                    }
                }
                KeyCode::Backspace => {
                    self.input_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.input_buffer.push(c);
                }
                _ => {}
            },
            Mode::SessionList => match key.code {
                KeyCode::Esc => {
                    self.mode = Mode::Normal;
                    self.status_message = "Normal mode".to_string();
                }
                _ => {}
            },
        }

        Ok(())
    }

    async fn create_session(&mut self) -> Result<()> {
        self.client.send(&ClientMessage::CreateSession {
            provider: "anthropic".to_string(),
            model: None,
        }).await?;

        self.status_message = "Creating session...".to_string();
        Ok(())
    }

    async fn list_sessions(&mut self) -> Result<()> {
        self.client.send(&ClientMessage::ListSessions).await?;
        self.mode = Mode::SessionList;
        self.status_message = "Loading sessions...".to_string();
        Ok(())
    }

    async fn send_message(&mut self) -> Result<()> {
        if let Some(ref session_id) = self.current_session {
            let content = self.input_buffer.clone();
            self.input_buffer.clear();

            // Add user message to display
            self.messages.push_back(DisplayMessage {
                role: "user".to_string(),
                content: content.clone(),
            });

            // Add placeholder for assistant response
            self.messages.push_back(DisplayMessage {
                role: "assistant".to_string(),
                content: String::new(),
            });

            self.client.send(&ClientMessage::SendMessage {
                session_id: session_id.clone(),
                content,
            }).await?;

            self.status_message = "Sending...".to_string();
        } else {
            self.status_message = "No active session. Press 'n' to create one.".to_string();
        }

        Ok(())
    }

    pub async fn poll(&mut self) -> Result<()> {
        if let Ok(Some(msg)) = self.client.try_recv().await {
            self.handle_server_message(msg);
        }

        Ok(())
    }

    fn handle_server_message(&mut self, msg: ServerMessage) {
        match msg {
            ServerMessage::Authenticated { user_id } => {
                self.authenticated = true;
                self.status_message = format!("Authenticated as {}", user_id);
            }
            ServerMessage::SessionCreated { session_id } => {
                self.current_session = Some(session_id.clone());
                self.status_message = format!("Session created: {}", session_id);
            }
            ServerMessage::Token { content, .. } => {
                // Append to last assistant message
                if let Some(last) = self.messages.back_mut() {
                    if last.role == "assistant" {
                        last.content.push_str(&content);
                    }
                }
            }
            ServerMessage::MessageComplete { .. } => {
                self.status_message = "Message complete".to_string();
            }
            ServerMessage::Error { code, message } => {
                self.status_message = format!("Error {}: {}", code, message);
            }
            ServerMessage::SessionsList { sessions } => {
                self.sessions = sessions;
                self.status_message = format!("Found {} sessions", self.sessions.len());
            }
            ServerMessage::SessionLoaded { session, messages } => {
                self.current_session = Some(session.id.clone());
                self.messages.clear();
                for msg in messages {
                    self.messages.push_back(DisplayMessage {
                        role: msg.role,
                        content: msg.content,
                    });
                }
                self.status_message = format!("Loaded session {}", session.id);
                self.mode = Mode::Normal;
            }
        }
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),      // Messages
                Constraint::Length(3),    // Input
                Constraint::Length(1),    // Status
            ])
            .split(frame.size());

        // Render messages
        self.render_messages(frame, chunks[0]);

        // Render input
        self.render_input(frame, chunks[1]);

        // Render status line
        self.render_status(frame, chunks[2]);
    }

    fn render_messages<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let messages: Vec<Line> = self.messages.iter().flat_map(|msg| {
            let mut lines = vec![
                Line::from(Span::styled(
                    format!("{}:", msg.role),
                    Style::default()
                        .fg(if msg.role == "user" { Color::Cyan } else { Color::Green })
                        .add_modifier(Modifier::BOLD),
                )),
            ];

            // Simple line-wrapped rendering (termimad would be better but requires more setup)
            for line in msg.content.lines() {
                lines.push(Line::from(line.to_string()));
            }

            lines.push(Line::from(""));
            lines
        }).collect();

        let paragraph = Paragraph::new(messages)
            .block(Block::default().borders(Borders::ALL).title("Conversation"))
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    fn render_input<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let input = Paragraph::new(self.input_buffer.as_str())
            .block(Block::default().borders(Borders::ALL).title("Input"));

        frame.render_widget(input, area);
    }

    fn render_status<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let session_info = if let Some(ref id) = self.current_session {
            format!(" | Session: {}", &id[..8])
        } else {
            " | No session".to_string()
        };

        let status = Line::from(vec![
            Span::styled(
                format!("{} {}", self.status_message, session_info),
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
        ]);

        let paragraph = Paragraph::new(status);
        frame.render_widget(paragraph, area);
    }
}
