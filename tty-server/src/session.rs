use anyhow::{Result, anyhow};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::config::Config;
use crate::db::Database;
use crate::providers::{Provider, ProviderMessage};

pub struct SessionManager {
    db: Database,
    config: Config,
    active_sessions: HashMap<String, ActiveSession>,
}

pub struct ActiveSession {
    pub session_id: String,
    pub provider_tx: mpsc::UnboundedSender<ProviderMessage>,
}

impl SessionManager {
    pub fn new(db: Database, config: Config) -> Self {
        Self {
            db,
            config,
            active_sessions: HashMap::new(),
        }
    }

    pub fn create_session(
        &mut self,
        user_id: &str,
        provider_name: &str,
        model: Option<String>,
    ) -> Result<String> {
        // Get provider config
        let provider_config = self.config.providers.get(provider_name)
            .ok_or_else(|| anyhow!("Provider {} not found", provider_name))?;

        let model = model.unwrap_or_else(|| {
            match provider_config {
                crate::config::ProviderConfig::Anthropic { default_model, .. } => default_model.clone(),
                crate::config::ProviderConfig::OpenAI { default_model, .. } => default_model.clone(),
                crate::config::ProviderConfig::OpenAICompatible { default_model, .. } => default_model.clone(),
            }
        });

        // Create session in database
        let session = self.db.create_session(user_id, provider_name, &model)?;

        // Create provider instance
        let (tx, rx) = mpsc::unbounded_channel();
        let provider = Provider::new(provider_config.clone())?;

        // Spawn provider task
        let session_id = session.id.clone();
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) = provider.run(session_id.clone(), rx, db).await {
                tracing::error!("Provider error in session {}: {}", session_id, e);
            }
        });

        // Store active session
        self.active_sessions.insert(
            session.id.clone(),
            ActiveSession {
                session_id: session.id.clone(),
                provider_tx: tx,
            },
        );

        Ok(session.id)
    }

    pub fn send_message(
        &self,
        session_id: &str,
        content: String,
        response_tx: mpsc::UnboundedSender<StreamEvent>,
    ) -> Result<()> {
        let active = self.active_sessions.get(session_id)
            .ok_or_else(|| anyhow!("Session {} not found or inactive", session_id))?;

        active.provider_tx.send(ProviderMessage::SendMessage {
            content,
            response_tx,
        })?;

        Ok(())
    }

    pub fn list_sessions(&self, user_id: &str) -> Result<Vec<crate::db::models::Session>> {
        self.db.list_sessions(user_id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<crate::db::models::Session>> {
        self.db.get_session(session_id)
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<crate::db::models::Message>> {
        self.db.get_messages(session_id)
    }

    pub fn delete_session(&mut self, session_id: &str) -> Result<()> {
        // Remove from active sessions
        if let Some(active) = self.active_sessions.remove(session_id) {
            let _ = active.provider_tx.send(ProviderMessage::Shutdown);
        }

        // Delete from database
        self.db.delete_session(session_id)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Token(String),
    Complete,
    Error(String),
}
