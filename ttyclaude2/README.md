# ttyclaude2 - Terminal Client

Fast, keyboard-driven terminal interface for LLM conversations. Connects to tty-server via native binary protocol.

## Features

- **Vim-Style Keybindings**
  - Normal mode for navigation
  - Insert mode for typing
  - Modal editing paradigm

- **Streaming Markdown**
  - Real-time token rendering
  - Syntax-highlighted code blocks
  - Tables, lists, formatting

- **Fast**
  - Sub-100ms startup (goal)
  - < 50MB memory footprint
  - Minimal dependencies

- **Session Management**
  - Create new conversations
  - Resume previous sessions
  - List all sessions

- **Local-First**
  - All data stored server-side
  - Client is stateless
  - Quick reconnection

## Installation

```bash
cargo build --release
sudo cp target/release/ttyclaude2 /usr/local/bin/
```

## Configuration

Create `~/.config/ttyclaude2/config.toml`:

```toml
[server]
address = "127.0.0.1:7331"  # or Unix socket path

[auth]
token = "default_user"  # Will be JWT in future

[ui]
vim_mode = true
theme = "monokai"
```

## Usage

```bash
ttyclaude2
```

### Keybindings

**Normal Mode**:
- `i` - Enter insert mode
- `n` - Create new session
- `l` - List sessions
- `Esc` - Ensure normal mode
- `Ctrl+C` - Quit

**Insert Mode**:
- `Enter` - Send message
- `Backspace` - Delete character
- `Esc` - Return to normal mode
- Type to compose message

**Session List Mode** (planned):
- `j/k` - Navigate up/down
- `Enter` - Load selected session
- `d` - Delete session
- `Esc` - Return to normal mode

## Architecture

```
ttyclaude2
├── TUI Layer (ratatui)
│   ├── Message rendering
│   ├── Input handling
│   └── Status line
├── Protocol Client
│   ├── Binary framing
│   ├── JSON serialization
│   └── Async I/O
└── Config Management
```

The client is intentionally thin. All business logic lives in tty-server.

## Development

```bash
# Build
cargo build

# Run
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Lint
cargo clippy

# Format
cargo fmt
```

## Markdown Rendering

Currently uses basic ratatui rendering. Planned upgrade to termimad for full markdown support:

- Code block syntax highlighting (syntect)
- Tables
- Nested lists
- Inline formatting (bold, italic, code)

## Remote Usage

### SSH Tunnel

```bash
# On local machine
ssh -L 7331:localhost:7331 user@vps

# Then run ttyclaude2 locally
ttyclaude2
```

### mosh + tmux

```bash
# Connect to VPS
mosh user@vps

# Start or attach tmux session
tmux new -s chat
# or
tmux attach -t chat

# Run ttyclaude2
ttyclaude2
```

## Comparison with Original ttyclaude

### Original (Monolithic)

- ✓ Simple single binary
- ✓ No network dependencies
- ✗ No session sharing
- ✗ Can't access from other devices
- ✗ Harder to add web interface

### ttyclaude2 (Client-Server)

- ✓ Session persistence on server
- ✓ Access from multiple devices
- ✓ Web client available
- ✓ Shared team deployment possible
- ✗ Requires running server
- ✗ Network latency (minimal on localhost)

## Philosophy

ttyclaude2 preserves the terminal-first philosophy while gaining flexibility:

- Keyboard-driven (vim bindings)
- Fast startup and low resource usage
- Works great in screen/tmux
- But now: sessions persist, work continues anywhere

## Roadmap

- [ ] Full vim keybindings (hjkl navigation, visual mode, etc.)
- [ ] Termimad integration for better markdown
- [ ] Session search
- [ ] Export to markdown
- [ ] Offline queue (compose while disconnected)
- [ ] Multiple simultaneous sessions (buffers/tabs)
- [ ] Custom color schemes
- [ ] Mouse support (optional)

## Troubleshooting

### Connection Failed

```bash
# Check if server is running
curl http://localhost:7332/health

# Check server logs
journalctl -u tty-server -f
```

### Rendering Issues

```bash
# Set TERM correctly
echo $TERM  # Should be xterm-256color or similar

# Test terminal capabilities
tput colors  # Should be 256
```

### Performance

```bash
# Profile with perf
cargo build --release
perf record --call-graph dwarf ./target/release/ttyclaude2
perf report
```

## License

See [LICENSE](../LICENSE)
