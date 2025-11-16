use crate::{Provider, SessionCommand, SessionEvent};
use tokio::sync::mpsc;

pub struct EchoProvider;

impl Provider for EchoProvider {
    async fn run(
        self,
        mut cmd_rx: mpsc::Receiver<SessionCommand>,
        evt_tx: mpsc::Sender<SessionEvent>,
    ) {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                SessionCommand::SendMessage(text) => {
                    // Simulate "streaming" by sending word by word
                    for word in text.split_whitespace() {
                        let _ = evt_tx.send(SessionEvent::TokenReceived(
                            format!("{} ", word)
                        )).await;
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    }
                    let _ = evt_tx.send(SessionEvent::ResponseComplete).await;
                }
                SessionCommand::Cancel => {
                    // Nothing streaming, ignore
                }
                SessionCommand::Shutdown => break,
            }
        }
    }
}