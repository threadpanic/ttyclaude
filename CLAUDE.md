# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ttyclaude** is a minimal, keyboard-driven terminal client for conversing with Claude AI and other LLMs. It's designed for Unix workflow integration with `screen`/`tmux`, enabling rapid context-switching between strategic conversation (ttyclaude) and tactical execution (Claude Code).

**Current Status**: Early development (v0.1 MVP phase). Basic "Hello World" binary exists; core features are being designed and planned.

## Build & Development Commands

```bash
# Build the project
cargo build
# Or use make
make

# Build for release (optimized)
cargo build --release

# Run the binary
cargo run

# Run tests
cargo test

# Lint with clippy
cargo clippy

# Format code
cargo fmt

# Clean build artifacts
cargo clean
# Or use make
make clean
```

## Architecture & Design Philosophy

### Core Design Principles

1. **Message Passing, Not Shared State** - Unix philosophy: processes communicate via channels (like pipes), not shared memory
2. **Minimal Dependencies** - Should build and run on any Unix-like system with minimal external requirements
3. **Keyboard-First** - All operations via keyboard; mouse is optional
4. **Fast** - Sub-100ms startup target, < 50MB memory footprint
5. **Local-First** - User data stored locally in sqlite, no telemetry

### Mental Model: Linear Narrative Threading

The architecture is designed around **linear, story-like cognition** rather than traditional multithreaded complexity:

```
Main Thread (Linear Narrative)
    ↓ channels (one-way message passing)
Session Threads (Isolated Workers)
    ↓ HTTP streaming (you don't care about details)
LLM APIs (External World)
```

**Key insight**: Main thread never blocks on network I/O. Session threads are isolated black boxes that receive commands and emit events via channels (`mpsc::channel`).

### Technology Stack

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }      # Async runtime
reqwest = { version = "0.11", features = ["stream"] } # HTTP client with SSE
crossterm = "0.27"                                   # Terminal manipulation
ratatui = "0.25"                                     # TUI framework
termimad = "0.23"                                    # Markdown rendering (core feature)
syntect = "5.1"                                      # Code syntax highlighting
rusqlite = "0.30"                                    # SQLite database
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"                                       # Error handling
```

### Multi-Provider Architecture

ttyclaude supports multiple LLM providers simultaneously, switchable with Alt-1/2/3/4:

1. **Commercial APIs** (Anthropic, OpenAI) - API key auth, pay per token
2. **Self-Hosted Remote** (lmcli on VPS, ollama) - OpenAI-compatible APIs on your infrastructure
3. **Local Inference** (llama.cpp on localhost) - No network, uses laptop resources

Each provider is treated identically through the OpenAI-compatible interface pattern.

### Code Structure (Planned)

```
src/
├── main.rs             # Entry point, CLI arg parsing
├── lib.rs              # Library interface (for testing)
├── api/
│   ├── mod.rs          # API module interface
│   ├── client.rs       # Anthropic & OpenAI-compatible clients
│   ├── streaming.rs    # SSE stream handling
│   └── models.rs       # API request/response types
├── ui/
│   ├── mod.rs
│   ├── app.rs          # Main TUI application loop (tokio::select!)
│   ├── events.rs       # Keyboard/terminal event handling
│   ├── renderer.rs     # Markdown rendering logic
│   └── widgets.rs      # Custom ratatui widgets
├── db/
│   ├── mod.rs
│   ├── schema.rs       # Database schema/migrations
│   ├── queries.rs      # SQL query functions
│   └── models.rs       # Database record types
└── config/
    ├── mod.rs
    └── settings.rs     # Config file parsing (~/.config/ttyclaude/config.toml)
```

## Feature Roadmap

### MVP (v0.1) - In Progress
- Basic TUI with input/output
- **Glow-style markdown rendering** (core feature, not optional)
- API client with streaming (Server-Sent Events)
- Simple conversation flow (single session, no persistence yet)
- Configuration from file
- Graceful error handling

### v0.2 - Persistence
- sqlite conversation storage
- List/load previous conversations
- Export to markdown
- Session resumption

### v0.3 - Enhanced UI
- Vim-style keybindings throughout
- Search within conversation
- Status line with token count, model info
- Custom color schemes

### v0.4 - Integration
- Pipe content into ttyclaude
- Shell command execution integration
- File attachments

### v0.5 - Advanced
- Conversation branching
- Multiple simultaneous conversations (tabs/buffers)
- Full-text search across all conversations

## Key Technical Challenges

### Streaming Markdown Rendering

**Challenge**: Display tokens as they arrive without jarring repositioning, while maintaining markdown rendering quality

**Approach**:
- Hybrid buffering: accumulate 50-100 char chunks
- Re-render markdown incrementally
- Gracefully handle incomplete markdown structures (mid-code-block, unclosed lists)
- Use double-buffering to prevent flicker

### Message Passing Pattern

```rust
// Command channel: Main thread -> Session thread
enum SessionCommand {
    SendMessage(String),
    Cancel,
    Shutdown,
}

// Event channel: Session thread -> Main thread
enum SessionEvent {
    TokenReceived(String),
    ResponseComplete,
    Error(String),
}

// Main loop (linear narrative)
loop {
    select! {
        key = read_key() => handle_input(key, &cmd_tx),
        evt = evt_rx.recv() => update_buffer(evt),
    }
}
```

No mutexes, no atomics, no memory fences. Just channels.

## Configuration

Location: `~/.config/ttyclaude/config.toml`

```toml
[api]
key = "sk-ant-..."  # or set $ANTHROPIC_API_KEY
model = "claude-sonnet-4-20250514"

[ui]
vim_mode = true
markdown_style = "auto"  # auto, dark, light, dracula
code_theme = "monokai"

[providers.vps-lmcli]
type = "openai-compatible"
endpoint = "https://my-vps.example.com:8080"
models = ["phi-4-mini-reasoning"]
default_model = "phi-4-mini-reasoning"
```

## Coding Standards

### Quality Gates
- `cargo build` - Must compile without warnings
- `cargo test` - All tests pass
- `cargo clippy` - No clippy warnings
- `cargo fmt --check` - Formatted correctly

### Code Style
- Follow Rust community conventions
- Prefer explicit over clever
- Document public APIs with `///` comments
- Keep functions small and focused
- Use `anyhow` for error handling in application code
- Use `Result<T>` for library code

### Async Patterns
- Use `tokio::spawn` for true concurrency
- Use `tokio::select!` for racing operations
- Channel patterns for message passing (main thread ↔ session threads)
- Avoid `.await` in hot loops when possible

## Important Context

### The Companion Workflow

ttyclaude is designed to work alongside Claude Code in a `screen`/`tmux` session:

```bash
screen -S work
# window 0: ttyclaude - strategy, planning, discussion
# window 1: claude - tactical execution, coding
# Ctrl-a 0/1 - flip between thinking and doing
```

### Markdown Rendering Is Core

This is not an optional nicety. Claude outputs markdown natively; code blocks, tables, lists need proper rendering. The tool uses:
- **termimad** - Markdown → styled terminal output (used by broot)
- **syntect** - Sublime Text's highlighting engine for code blocks

### Multi-Provider Philosophy

Different LLMs for different contexts:
- **Anthropic Opus**: Hard problems, architecture decisions (API cost justified)
- **VPS phi-4**: Code review, quick questions (unlimited, fast, private)
- **Local mistral**: Offline work, truly private
- **OpenAI GPT-4**: Second opinions, team compatibility

Alt-N to switch. Right tool for the right job.

## Development Workflow

This is the authors' first substantial Rust project published on Codeberg. Expect iteration and learning.

**Development Loop**:
1. Design/plan feature (update DESIGN.md if architectural)
2. Write tests first (TDD when practical)
3. Implement in small chunks
4. Run `cargo clippy` - fix all warnings
5. Run `cargo fmt`
6. Test manually in real terminal/screen/tmux
7. Commit with descriptive messages

## References

- [DESIGN.md](DESIGN.md) - Full architecture, technical decisions, roadmap
- [DESIGN2.md](DESIGN2.md) - VPS & self-hosted LLM deployment patterns
- [mental_model.md](mental_model.md) - Threading mental model (message passing vs shared state)
- [Anthropic API Docs](https://docs.anthropic.com/en/api/)
- [Ratatui Book](https://ratatui.rs/tutorial/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
