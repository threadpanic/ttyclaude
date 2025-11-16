use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use tokio::sync::mpsc;
use tokio::select;

mod providers;
use providers::{echo::EchoProvider, shell::ShellProvider, Provider, SessionCommand, SessionEvent};

#[tokio::main]
async fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Clear screen before first draw
    terminal.clear()?;

    // Spawn session - your mental model in action
    let (cmd_tx, cmd_rx) = mpsc::channel::<SessionCommand>(32);
    let (evt_tx, mut evt_rx) = mpsc::channel::<SessionEvent>(32);

    // Choose provider from environment or default to Echo
    let provider_name = std::env::var("PROVIDER").unwrap_or_else(|_| "echo".to_string());
    let active_provider = match provider_name.to_lowercase().as_str() {
        "shell" => {
            tokio::spawn(ShellProvider.run(cmd_rx, evt_tx));
            "Shell"
        }
        _ => {
            tokio::spawn(EchoProvider.run(cmd_rx, evt_tx));
            "Echo"
        }
    };

    let mut input_buffer = String::new();
    let mut response_buffer = String::new();
    let mut should_quit = false;

    while !should_quit {
        // Draw UI with current buffers
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.area());

            let title = Paragraph::new(format!(
                "ttyclaude - {} Provider (q=quit) | Set PROVIDER=shell or PROVIDER=echo",
                active_provider
            ))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(title, chunks[0]);

            let response = Paragraph::new(response_buffer.as_str())
                .block(Block::default().borders(Borders::ALL).title("Response"));
            f.render_widget(response, chunks[1]);

            let input = Paragraph::new(format!("> {}", input_buffer))
                .block(Block::default().borders(Borders::ALL).title("Input (Enter to send)"));
            f.render_widget(input, chunks[2]);
        })?;

        // The select! - your linear story with mail checks
        select! {
            // Check for keyboard input (non-blocking)
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                if event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        match key.code {
                            KeyCode::Char('q') => should_quit = true,
                            KeyCode::Enter => {
                                // Send message, clear input
                                let _ = cmd_tx.send(SessionCommand::SendMessage(
                                    input_buffer.clone()
                                )).await;
                                input_buffer.clear();
                            }
                            KeyCode::Char(c) => input_buffer.push(c),
                            KeyCode::Backspace => { input_buffer.pop(); }
                            _ => {}
                        }
                    }
                }
            }
            // Check for session events
            Some(evt) = evt_rx.recv() => {
                match evt {
                    SessionEvent::TokenReceived(token) => {
                        response_buffer.push_str(&token);
                    }
                    SessionEvent::ResponseComplete => {
                        response_buffer.push_str("\n--- done ---\n");
                    }
                    SessionEvent::Error(e) => {
                        response_buffer.push_str(&format!("\nERROR: {}\n", e));
                    }
                }
            }
        }
    }

    // Cleanup
    let _ = cmd_tx.send(SessionCommand::Shutdown).await;
    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
