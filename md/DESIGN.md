# ttyclaude - Design Document

## Vision

A minimal, keyboard-driven terminal client for conversing with Claude that integrates seamlessly into Unix workflows. Designed for the use case of running in a `screen`/`tmux` session alongside `claude` (Claude Code), enabling rapid context-switching between strategic conversation (ttyclaude) and tactical execution (coding agent).

### Core Use Case
```bash
screen -S work
# window 0: ttyclaude - planning, discussion, thinking
# window 1: claude - execution, coding  
# Ctrl-a 0/1 to flip between strategist and executor
```

## Design Principles

1. **Minimal Dependencies** - Should build and run on any Unix-like system with minimal external requirements
2. **Fast Startup** - Sub-100ms cold start, instant warm start
3. **Keyboard-First** - Mouse is optional or nonexistent; all operations via keyboard
4. **Scriptable I/O** - Support piping content in/out, shell integration
5. **Screen/Tmux Native** - Perfect integration with terminal multiplexers
6. **Local-First** - User owns their data, conversations stored locally in sqlite
7. **No Bloat** - Every feature must justify its complexity cost

## Architecture

### High-Level Components

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚         Terminal UI Layer               â”‚
â”‚  (input handling, display, rendering)   â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
              â”‚
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚      Application Core                   â”‚
â”‚  (conversation state, message queue)    â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¬â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
              â”‚
    â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
    â”‚                   â”‚
â”Œâ”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”    â”Œâ”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚ API Client â”‚    â”‚  Local Store  â”‚
â”‚ (Anthropic)â”‚    â”‚  (sqlite)     â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜    â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Component Details

#### Terminal UI Layer
- **Responsibilities**: Input capture, markdown rendering, keyboard event handling
- **Markdown Rendering**: Beautiful markdown display (core feature, not optional)
  - Syntax highlighted code blocks (syntect)
  - Proper list formatting, headers, emphasis, links
  - Block quotes, tables
  - Should look as good as viewing in a quality terminal markdown viewer
- **Technology Stack**:
  - **crossterm** - Cross-platform terminal manipulation (raw mode, events)
  - **ratatui** - High-level TUI framework (layouts, widgets)
  - **termimad** - Markdown rendering with terminal styling
  - **syntect** - Code syntax highlighting (Sublime Text engine)
- **Key Bindings**: Vim-style where appropriate, standard readline elsewhere
- **Display Zones**:
  - Conversation history (scrollable, rendered markdown)
  - Input area (multiline support, raw text)
  - Status line (model, token count, connection status)
  - Command mode (vim-style `:` commands)

#### Application Core
- **Conversation State Machine**:
  - Idle â†’ Waiting for input
  - Composing â†’ User typing
  - Sending â†’ API request in flight
  - Streaming â†’ Receiving streamed response
  - Error â†’ Display error, return to idle
- **Message Queue**: Handle async API responses without blocking UI
- **Session Management**: Track conversation context, handle multi-turn conversations

#### API Client
- **Streaming Support**: Handle Server-Sent Events (SSE) for real-time responses
- **Rate Limiting**: Respect API limits, queue requests if needed
- **Error Handling**: Network failures, API errors, token limits
- **Authentication**: API key from config file or environment variable
- **Model Selection**: Support switching between Claude models (Sonnet, Opus, etc.)

#### Local Store (sqlite)
- **Schema**:
  ```sql
  conversations (
    id INTEGER PRIMARY KEY,
    title TEXT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
  )
  
  messages (
    id INTEGER PRIMARY KEY,
    conversation_id INTEGER,
    role TEXT, -- 'user' or 'assistant'
    content TEXT,
    timestamp TIMESTAMP,
    model TEXT,
    tokens_used INTEGER,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
  )
  
  config (
    key TEXT PRIMARY KEY,
    value TEXT
  )
  ```
- **Conversation History**: Full searchable history
- **Resumption**: Ability to continue previous conversations
- **Export**: Easy export to markdown, JSON, etc.

## Technical Decisions

### Language Choice

**Considerations**:
- **C++**: Maximum control, familiar to us, but lots of boilerplate for async/HTTP
- **Rust**: Modern, safe, excellent async and HTTP libraries (tokio, reqwest), compile-time guarantees
- **Python**: Rapid prototyping, easier API handling, but slower startup
- **Go**: Fast compilation, good concurrency, simpler than Rust

**Decision: Rust**

Rationale:
- **Existing knowledge**: We already know Rust, no learning curve
- **Cargo**: Best-in-class dependency management, zero dependency hell
- **Mature ecosystem**: All needed crates are battle-tested
- **Type safety**: Catch bugs at compile time
- **Performance**: Native performance, small binaries
- **Async/streaming**: tokio + reqwest handle SSE perfectly

**Compilation Speed**:
- Clean build: ~15-30 seconds (one-time)
- Incremental: ~3-5 seconds (typical during development)
- `cargo check`: ~1-2 seconds (quick validation)
- Not as fast as Go, but acceptable for this project size

**Technology Stack**:
```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.11", features = ["stream"] }
crossterm = "0.27"           # Terminal manipulation
ratatui = "0.25"             # TUI framework (tui-rs successor)
termimad = "0.23"            # Markdown rendering for terminals
syntect = "5.1"              # Syntax highlighting (Sublime quality)
rusqlite = "0.30"            # SQLite database
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"               # Error handling
```

**Markdown Rendering**:
- **termimad**: Proven markdown â†’ terminal renderer (used by broot)
- **syntect**: Sublime Text's highlighting engine (supports 100+ languages)
- Won't be pixel-identical to Glow, but gorgeous and more than sufficient
- Full control over styling and customization

### Configuration

**Location**: `~/.config/ttyclaude/config.toml`

```toml
[api]
key = "sk-ant-..."  # or from $ANTHROPIC_API_KEY
model = "claude-sonnet-4-20250514"
max_tokens = 4096
temperature = 1.0

[ui]
theme = "default"  # or "dark", "light", custom
vim_mode = true
show_token_count = true
markdown_style = "auto"  # auto, dark, light, notty, pink, dracula
code_theme = "monokai"  # syntax highlighting theme for code blocks
word_wrap = true
pager_style = true  # scroll like less/more

[storage]
db_path = "~/.local/share/ttyclaude/conversations.db"
auto_save = true
```

### Key Bindings (Initial Proposal)

**Normal Mode** (viewing conversation):
- `i` - Enter insert mode (compose message)
- `j/k` - Scroll up/down
- `g/G` - Jump to top/bottom
- `/` - Search in conversation
- `r` - Toggle raw/rendered markdown view
- `:` - Enter command mode
- `Ctrl-c` - Interrupt streaming response
- `q` - Quit (with confirmation)

**Insert Mode** (composing message):
- `Esc` - Return to normal mode
- `Ctrl-d` - Send message
- `Ctrl-c` - Cancel composition
- `Alt-Enter` - Insert newline (for multiline messages)

**Command Mode**:
- `:q` - Quit
- `:w [file]` - Export conversation
- `:new` - New conversation
- `:list` - List recent conversations
- `:load <id>` - Load conversation by ID
- `:model <name>` - Switch model
- `:clear` - Clear screen (keep conversation)
- `:reset` - Start fresh conversation

### Streaming Display

**Challenge**: Display tokens as they arrive without jarring repositioning, while maintaining markdown rendering

**Approach**:
1. Reserve bottom portion of screen for active assistant response
2. As tokens stream in, buffer them
3. Re-render markdown in real-time (Glamour can handle incremental updates)
4. When response complete, move to scrollback and clear active area
5. Use double-buffering to prevent flicker

**Visual Indicator**:
```
[User] How does async work in Rust?

[Claude] â–Œ (cursor blinks while streaming)
```

Then as markdown streams in, render it beautifully:

```
[Claude]
  Async in Rust is built on the Future trait. Here's how it works:

  â€¢ Futures are lazy - they don't run until polled
  â€¢ async/await syntax makes writing async code ergonomic
  â€¢ The runtime (tokio, async-std) drives execution

  Example code:

  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
  â”‚ async fn fetch_data() -> Result<()> { â”‚
  â”‚     let data = get_url().await?;       â”‚
  â”‚     process(data).await                â”‚
  â”‚ }                                      â”‚
  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Markdown Rendering Details

**Why Beautiful Markdown?**
- We work in markdown daily
- Claude outputs markdown natively
- Code blocks need proper syntax highlighting
- Tables, lists, quotes look beautiful when rendered
- Keeps us in terminal but makes it pleasant

**Rust Implementation**:
- **termimad** - Markdown â†’ styled terminal output
  - Handles tables, lists, quotes, code blocks
  - Supports custom styling/colors
  - Used by broot and other quality terminal tools
  - Flexible and battle-tested
- **syntect** - Syntax highlighting for code blocks
  - Same engine as Sublime Text
  - Supports 100+ languages out of box
  - Themes: Monokai, Solarized, Dracula, etc.
  - Can load custom .tmTheme files
- **Custom rendering pipeline**:
  1. Parse markdown (pulldown-cmark under the hood via termimad)
  2. Apply syntax highlighting to code blocks (syntect)
  3. Render to ANSI with termimad styling
  4. Display in ratatui widget

**Streaming Markdown Challenge**:
Incomplete markdown (mid-code-block, unclosed lists) needs graceful handling:
- Option 1: Buffer until markdown is "complete enough" (100-200 chars?)
- Option 2: Show raw text while streaming, render when complete
- Option 3: Incremental render with "best effort" (may flicker)
- **Recommendation**: Hybrid - buffer small amounts (smooth), render frequently (responsive)
  - Buffer tokens in 50-100 char chunks
  - Try to break on word/sentence boundaries
  - Re-render markdown on each chunk
  - Gracefully handle incomplete structures

## Feature Roadmap

### MVP (v0.1)
- [x] Project setup
- [ ] Basic TUI with input/output
- [ ] **Glow-style markdown rendering** (core feature)
- [ ] API client with streaming
- [ ] Simple conversation flow (single session, no persistence)
- [ ] Configuration from file
- [ ] Graceful error handling

### v0.2 - Persistence
- [ ] sqlite conversation storage
- [ ] List/load previous conversations
- [ ] Export to markdown (raw or rendered)
- [ ] Session resumption

### v0.3 - Enhanced UI
- [ ] Advanced syntax highlighting (more languages)
- [ ] Vim-style keybindings throughout
- [ ] Search within conversation (with highlight)
- [ ] Status line with useful info
- [ ] Custom color schemes/themes

### v0.4 - Integration
- [ ] Pipe content into ttyclaude
- [ ] Copy assistant responses to clipboard
- [ ] Shell command execution integration
- [ ] Support for file attachments

### v0.5 - Advanced
- [ ] Conversation branching (fork from any point)
- [ ] Multiple simultaneous conversations (tabs?)
- [ ] Local full-text search across all conversations
- [ ] Customizable themes/colors

### Future Ideas
- Tool use display (show when Claude calls functions)
- Integration with local LLMs for comparison
- RAG over conversation history
- Conversation templates/prompts library
- Split-pane mode (show code/docs while chatting)

## Non-Goals

- **GUI version** - Terminal only, forever
- **Web interface** - Use claude.ai for that
- **Mobile support** - This is a workstation tool
- **Complex threading/rooms** - Keep it simple, one conversation at a time
- **Multimedia** - Text only (though maybe image display in terminals that support it?)

## Performance Targets

- **Cold start**: < 100ms
- **Input latency**: < 16ms (60fps feel)
- **First token**: < 1s after send (network dependent)
- **Memory footprint**: < 50MB resident
- **Conversation history**: Support 10k+ messages without slowdown

## Security Considerations

- **API Key Storage**: File permissions 0600 on config
- **Never log API keys**: Sanitize all debug output
- **Secure defaults**: Warn if config file has loose permissions
- **No telemetry**: Zero phone-home, all local

## Development Workflow

This is our first substantial Rust project, so we'll iterate and learn as we go.

**Development Loop**:
1. Design/plan feature in this document
2. Write tests first (TDD when practical)
3. Implement in small, reviewable chunks
4. Run `cargo clippy` - fix all warnings
5. Run `cargo fmt` - consistent style
6. Test manually in real terminal
7. Commit with descriptive messages

**Quality Gates**:
- `cargo build` - Must compile without warnings
- `cargo test` - All tests pass
- `cargo clippy` - No clippy warnings
- `cargo fmt --check` - Formatted correctly
- Manual testing - Actually use it in screen/tmux

**Code Style**:
- Follow Rust community conventions
- Prefer explicit over clever
- Document public APIs with `///` comments
- Keep functions small and focused
- Use `anyhow` for error handling in application code
- Use `Result<T>` for library code

**Async Patterns**:
- Use `tokio::spawn` for true concurrency
- Use `tokio::select!` for racing operations
- Channel patterns for message passing
- Avoid `.await` in hot loops when possible

## Testing Strategy

- **Unit tests**: Core logic, API client, message parsing
- **Integration tests**: API interaction (with mocked responses)
- **Manual testing**: Actual usage in target environments
- **Fuzz testing**: Input handling, edge cases
- **Performance profiling**: Ensure we hit performance targets

## Build & Distribution

- **Build system**: Cargo
- **Target platforms**: Linux, macOS, BSD (anywhere Rust compiles)
- **Dependencies**: All from crates.io, no git dependencies
- **Packaging**: 
  - Source tarball
  - `cargo install` from crates.io (once published)
  - Statically linked binaries for major platforms (x86_64, aarch64)
  - Package manager recipes (homebrew, cargo, AUR, nixpkgs)
- **Documentation**: Man page, extensive README, `--help` output
- **Build profiles**:
  - Debug: Fast compile, helpful errors
  - Release: `--release` with LTO for smaller/faster binary
  - Minimal: Strip debug symbols, optimize for size

## Project Structure

```
ttyclaude/
â”œâ”€â”€ Cargo.toml              # Project manifest
â”œâ”€â”€ Cargo.lock              # Dependency lockfile
â”œâ”€â”€ README.md
â”œâ”€â”€ DESIGN.md
â”œâ”€â”€ LICENSE
â”œâ”€â”€ .gitignore
â”œâ”€â”€ src/
â”‚   â”œâ”€â”€ main.rs             # Entry point, CLI arg parsing
â”‚   â”œâ”€â”€ lib.rs              # Library interface (for testing)
â”‚   â”œâ”€â”€ api/
â”‚   â”‚   â”œâ”€â”€ mod.rs          # API module interface
â”‚   â”‚   â”œâ”€â”€ client.rs       # Anthropic API client
â”‚   â”‚   â”œâ”€â”€ streaming.rs    # SSE stream handling
â”‚   â”‚   â””â”€â”€ models.rs       # API request/response types
â”‚   â”œâ”€â”€ ui/
â”‚   â”‚   â”œâ”€â”€ mod.rs
â”‚   â”‚   â”œâ”€â”€ app.rs          # Main TUI application loop
â”‚   â”‚   â”œâ”€â”€ events.rs       # Keyboard/terminal event handling
â”‚   â”‚   â”œâ”€â”€ renderer.rs     # Markdown rendering logic
â”‚   â”‚   â””â”€â”€ widgets.rs      # Custom ratatui widgets
â”‚   â”œâ”€â”€ db/
â”‚   â”‚   â”œâ”€â”€ mod.rs
â”‚   â”‚   â”œâ”€â”€ schema.rs       # Database schema/migrations
â”‚   â”‚   â”œâ”€â”€ queries.rs      # SQL query functions
â”‚   â”‚   â””â”€â”€ models.rs       # Database record types
â”‚   â”œâ”€â”€ config/
â”‚   â”‚   â”œâ”€â”€ mod.rs
â”‚   â”‚   â””â”€â”€ settings.rs     # Config file parsing/validation
â”‚   â””â”€â”€ util/
â”‚       â”œâ”€â”€ mod.rs
â”‚       â””â”€â”€ markdown.rs     # Markdown utilities
â”œâ”€â”€ tests/
â”‚   â”œâ”€â”€ integration_test.rs
â”‚   â””â”€â”€ fixtures/           # Test data
â”œâ”€â”€ benches/                # Performance benchmarks
â”‚   â””â”€â”€ rendering_bench.rs
â””â”€â”€ examples/               # Usage examples
    â””â”€â”€ basic_chat.rs
```

## Success Metrics

This project succeeds if:
1. Daily users (us) prefer it over web interface for technical work
2. Integrates seamlessly into existing screen/tmux workflows  
3. Feels faster and more natural than switching to browser
4. Other terminal-loving devs adopt it
5. Codebase stays maintainable (< 5k LOC core)

## Open Questions

1. **Incremental markdown rendering**: How to smoothly update rendered markdown during streaming? Buffer size? Re-render frequency?
2. **Message format**: Store raw API JSON or normalize to internal format?
3. **Multiline input**: Modal (vim-style `i` to enter) or always-on multiline?
4. **Raw vs rendered toggle**: Let user see raw markdown with `r` keybind?
5. **Conversation titles**: Auto-generate from first message or user-provided?
6. **Context management**: How to handle long conversations hitting token limits?
   - Automatic summarization?
   - Manual pruning UI?
   - Warning at 75% of context?
7. **Markdown in input**: Should user input also be markdown-rendered in history?
8. **Syntax theme selection**: Ship with 2-3 themes or support loading custom .tmTheme?
9. **Error display**: How to show API errors without breaking immersion?

## References

**API & Documentation**:
- [Anthropic API Docs](https://docs.anthropic.com/en/api/)
- [Claude Sonnet 4 Model Card](https://www-cdn.anthropic.com/fed9cc193a14b84131812372d8d5857f8f304c52/Model_Card_Claude_4.pdf)

**Rust Crates**:
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [ratatui](https://ratatui.rs/) - TUI framework
- [crossterm](https://docs.rs/crossterm/) - Terminal manipulation
- [termimad](https://docs.rs/termimad/) - Markdown rendering
- [syntect](https://docs.rs/syntect/) - Syntax highlighting
- [rusqlite](https://docs.rs/rusqlite/) - SQLite bindings

**Inspiration**:
- [Glow](https://github.com/charmbracelet/glow) - Markdown renderer
- [broot](https://github.com/Canop/broot) - File manager using termimad
- irssi: <https://irssi.org/>
- weechat: <https://weechat.org/>

**Learning Resources**:
- [Ratatui Book](https://ratatui.rs/tutorial/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

---

**Last Updated**: 2025-11-06  
**Authors**: Threadpanic & Claude  
**License**: TBD (suggest GPL-3.0 or MIT)