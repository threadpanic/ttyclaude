

pub mod echo;
pub mod shell;



use tokio::sync::mpsc;

// The messages - same for ALL providers
#[derive(Debug)]
pub enum SessionCommand {
    SendMessage(String),
    Cancel,
    Shutdown,
}

#[derive(Debug)]
pub enum SessionEvent {
    TokenReceived(String),
    ResponseComplete,
    Error(String),
}

// Provider trait - what every provider must do
pub trait Provider: Send + 'static {
    async fn run(
        self,
        cmd_rx: mpsc::Receiver<SessionCommand>,
        evt_tx: mpsc::Sender<SessionEvent>,
    );
}

// Type tag for subsetting later (Stage 2)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderType {
    Echo,
    Shell,
    // Future: Anthropic, OpenAICompat, LlamaCpp
}
