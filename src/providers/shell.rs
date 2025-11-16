use crate::{Provider, SessionCommand, SessionEvent};
use tokio::sync::mpsc;
use tokio::process::Command;

pub struct ShellProvider;

impl Provider for ShellProvider {
    async fn run(
        self,
        mut cmd_rx: mpsc::Receiver<SessionCommand>,
        evt_tx: mpsc::Sender<SessionEvent>,
    ) {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                SessionCommand::SendMessage(text) => {
                    // Execute as shell command
                    match Command::new("sh")
                        .arg("-c")
                        .arg(&text)
                        .output()
                        .await
                    {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            
                            if !stdout.is_empty() {
                                let _ = evt_tx.send(SessionEvent::TokenReceived(
                                    stdout.to_string()
                                )).await;
                            }
                            if !stderr.is_empty() {
                                let _ = evt_tx.send(SessionEvent::TokenReceived(
                                    format!("stderr: {}", stderr)
                                )).await;
                            }
                            let _ = evt_tx.send(SessionEvent::ResponseComplete).await;
                        }
                        Err(e) => {
                            let _ = evt_tx.send(SessionEvent::Error(e.to_string())).await;
                        }
                    }
                }
                SessionCommand::Cancel => {}
                SessionCommand::Shutdown => break,
            }
        }
    }
}