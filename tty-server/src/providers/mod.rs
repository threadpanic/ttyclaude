use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use reqwest::Client;
use serde_json::json;

use crate::config::ProviderConfig;
use crate::db::Database;
use crate::session::StreamEvent;

pub enum ProviderMessage {
    SendMessage {
        content: String,
        response_tx: mpsc::UnboundedSender<StreamEvent>,
    },
    Shutdown,
}

pub struct Provider {
    config: ProviderConfig,
    client: Client,
}

impl Provider {
    pub fn new(config: ProviderConfig) -> Result<Self> {
        Ok(Self {
            config,
            client: Client::new(),
        })
    }

    pub async fn run(
        self,
        session_id: String,
        mut rx: mpsc::UnboundedReceiver<ProviderMessage>,
        db: Database,
    ) -> Result<()> {
        while let Some(msg) = rx.recv().await {
            match msg {
                ProviderMessage::SendMessage { content, response_tx } => {
                    // Save user message
                    db.add_message(&session_id, "user", &content)?;

                    // Get conversation history
                    let messages = db.get_messages(&session_id)?;

                    // Send to provider and stream response
                    if let Err(e) = self.stream_completion(&session_id, messages, response_tx, &db).await {
                        tracing::error!("Error streaming completion: {}", e);
                    }
                }
                ProviderMessage::Shutdown => {
                    tracing::info!("Provider shutting down for session {}", session_id);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn stream_completion(
        &self,
        session_id: &str,
        messages: Vec<crate::db::models::Message>,
        response_tx: mpsc::UnboundedSender<StreamEvent>,
        db: &Database,
    ) -> Result<()> {
        match &self.config {
            ProviderConfig::Anthropic { api_key, .. } => {
                self.stream_anthropic(session_id, messages, response_tx, db, api_key).await
            }
            ProviderConfig::OpenAI { api_key, .. } => {
                self.stream_openai(session_id, messages, response_tx, db, api_key, "https://api.openai.com/v1").await
            }
            ProviderConfig::OpenAICompatible { endpoint, api_key, .. } => {
                let key = api_key.as_deref().unwrap_or("none");
                self.stream_openai(session_id, messages, response_tx, db, key, endpoint).await
            }
        }
    }

    async fn stream_anthropic(
        &self,
        session_id: &str,
        messages: Vec<crate::db::models::Message>,
        response_tx: mpsc::UnboundedSender<StreamEvent>,
        db: &Database,
        api_key: &str,
    ) -> Result<()> {
        use eventsource_stream::Eventsource;
        use futures_util::StreamExt;

        let model = match &self.config {
            ProviderConfig::Anthropic { default_model, .. } => default_model,
            _ => return Err(anyhow!("Invalid provider config")),
        };

        // Format messages for Anthropic API
        let api_messages: Vec<_> = messages.iter()
            .map(|m| json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();

        let payload = json!({
            "model": model,
            "messages": api_messages,
            "max_tokens": 4096,
            "stream": true
        });

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let mut stream = response.bytes_stream().eventsource();
        let mut full_response = String::new();

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.event == "content_block_delta" {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                            if let Some(text) = data["delta"]["text"].as_str() {
                                full_response.push_str(text);
                                let _ = response_tx.send(StreamEvent::Token(text.to_string()));
                            }
                        }
                    } else if event.event == "message_stop" {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    let _ = response_tx.send(StreamEvent::Error(e.to_string()));
                    return Err(anyhow!("Stream error: {}", e));
                }
            }
        }

        // Save assistant response
        db.add_message(session_id, "assistant", &full_response)?;
        let _ = response_tx.send(StreamEvent::Complete);

        Ok(())
    }

    async fn stream_openai(
        &self,
        session_id: &str,
        messages: Vec<crate::db::models::Message>,
        response_tx: mpsc::UnboundedSender<StreamEvent>,
        db: &Database,
        api_key: &str,
        endpoint: &str,
    ) -> Result<()> {
        use eventsource_stream::Eventsource;
        use futures_util::StreamExt;

        let model = match &self.config {
            ProviderConfig::OpenAI { default_model, .. } => default_model,
            ProviderConfig::OpenAICompatible { default_model, .. } => default_model,
            _ => return Err(anyhow!("Invalid provider config")),
        };

        // Format messages for OpenAI API
        let api_messages: Vec<_> = messages.iter()
            .map(|m| json!({
                "role": m.role,
                "content": m.content
            }))
            .collect();

        let payload = json!({
            "model": model,
            "messages": api_messages,
            "stream": true
        });

        let url = format!("{}/chat/completions", endpoint);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let mut stream = response.bytes_stream().eventsource();
        let mut full_response = String::new();

        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data == "[DONE]" {
                        break;
                    }

                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(text) = data["choices"][0]["delta"]["content"].as_str() {
                            full_response.push_str(text);
                            let _ = response_tx.send(StreamEvent::Token(text.to_string()));
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Stream error: {}", e);
                    let _ = response_tx.send(StreamEvent::Error(e.to_string()));
                    return Err(anyhow!("Stream error: {}", e));
                }
            }
        }

        // Save assistant response
        db.add_message(session_id, "assistant", &full_response)?;
        let _ = response_tx.send(StreamEvent::Complete);

        Ok(())
    }
}
