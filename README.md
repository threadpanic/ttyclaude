# ttyclaude

A minimal, keyboard-driven terminal client for conversing with Claude AI. Designed for Unix workflow integration, particularly with `screen`/`tmux` and Claude Code.

## Why?

Because switching to a browser breaks flow. Because your terminal is your home. Because `screen ttyclaude` should be as natural as `vim config.toml`.

**The workflow:**
```bash
screen -S work
# window 0: ttyclaude - strategy, planning, discussion
# window 1: claude - tactical execution, coding
# Ctrl-a 0/1 - flip between thinking and doing
```

## Status

ðŸš§ **Early Development** - Design phase, not yet functional

This is a learning project - our first substantial Rust application published on Codeberg. Expect iteration, refactoring, and occasional "interesting" architectural decisions as we learn.

See [DESIGN.md](DESIGN.md) for full architecture and roadmap.

## Features (Planned)

- **Native Terminal UI** - No browser, no Electron, no BS
- **Glow-style Markdown** - Beautiful rendering like Glow itself (because we're markdown natives)
- **Streaming Responses** - Real-time token-by-token display with live markdown rendering
- **Local Storage** - Your conversations, your disk, sqlite
- **Vim Bindings** - Because of course
- **Scriptable** - Pipe content in/out, integrate with shell
- **Fast** - Sub-100ms startup, < 50MB memory
- **Private** - No telemetry, no phone-home

### Why Markdown Rendering Matters

Claude speaks markdown. We write markdown. Code blocks, tables, lists - they all look gorgeous when properly rendered. Why settle for raw text when you can have the Glow experience right in your conversation?

```
Instead of this:           You get this:
## Heading                â•”â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•—
- item one                â•‘ Heading          â•‘
- item two                â•šâ•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•â•
```rust                   â€¢ item one
fn main() {}              â€¢ item two
```                       
                          â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
                          â”‚ fn main() {}    â”‚
                          â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

## Prerequisites

- Unix-like OS (Linux, BSD, macOS)
- Rust toolchain (1.70+) - Install from [rustup.rs](https://rustup.rs)
- Anthropic API key - Get from [console.anthropic.com](https://console.anthropic.com)

## Installation

```bash
# Clone
git clone https://codeberg.org/Threadpanic/ttyclaude
cd ttyclaude

# Build
cargo build --release

# Install (optional)
cargo install --path .

# Or copy binary
cp target/release/ttyclaude ~/.local/bin/
```

## Configuration

Create `~/.config/ttyclaude/config.toml`:

```toml
[api]
key = "sk-ant-..."  # or set $ANTHROPIC_API_KEY
model = "claude-sonnet-4-20250514"

[ui]
vim_mode = true
markdown_style = "auto"  # auto, dark, light, dracula, pink
code_theme = "monokai"  # syntax highlighting for code blocks
```

Or use environment variable:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

## Usage

```bash
# Start new conversation
ttyclaude

# Resume last conversation
ttyclaude --continue

# Load specific conversation
ttyclaude --load 42

# Pipe content in
cat design.md | ttyclaude "review this design"
```

### Key Bindings

**Normal Mode** (viewing):
- `i` - Insert mode (compose message)
- `j`/`k` - Scroll
- `/` - Search
- `r` - Toggle raw/rendered markdown
- `:` - Command mode
- `Ctrl-c` - Interrupt response
- `q` - Quit

**Insert Mode** (composing):
- `Esc` - Back to normal mode
- `Ctrl-d` - Send message
- `Alt-Enter` - Newline

**Commands**:
- `:q` - Quit
- `:w file` - Export conversation
- `:new` - New conversation
- `:list` - Recent conversations
- `:model sonnet` - Switch model

## Roadmap

- **v0.1** - MVP: Basic TUI, streaming, single session
- **v0.2** - Persistence: sqlite storage, history
- **v0.3** - Enhanced UI: syntax highlighting, search
- **v0.4** - Integration: pipes, clipboard, files
- **v0.5** - Advanced: branching, tabs, themes

See [DESIGN.md](DESIGN.md) for details.

## Design Philosophy

1. **Minimal dependencies** - Standard Unix tools mindset
2. **Keyboard-first** - Mouse is a crutch
3. **Fast** - Instant response, no bloat
4. **Scriptable** - Play nice with pipes and scripts
5. **Local-first** - Your data stays yours
6. **No surprises** - Do what you expect, nothing more

## Comparison

| Feature | ttyclaude | Web UI | Claude Code |
|---------|-----------|--------|-------------|
| Terminal native | âœ“ | âœ— | âœ“ |
| Glow-style markdown | âœ“ | âœ“ | âœ— |
| Conversation | âœ“ | âœ“ | âœ— |
| Code execution | âœ— | âœ“ | âœ“ |
| Vim bindings | âœ“ | âœ— | âœ— |
| Local storage | âœ“ | Cloud | Local |
| Scriptable I/O | âœ“ | âœ— | âœ“ |
| Screen/tmux native | âœ“ | âœ— | âœ“ |

**Use ttyclaude for**: Planning, discussion, design, thinking  
**Use Claude Code for**: Implementation, debugging, refactoring  
**Use Web UI for**: Artifacts, images, complex interactions

## ttyclaude2 Quick Start

For the distributed version:

```bash
# 1. Start session server
cd tty-server
cargo run --release

# 2a. Use terminal client
cd ttyclaude2
cargo run --release

# 2b. Or use web client
cd tty-web
npm install && npm run dev
# Open http://localhost:5173
```

See individual READMEs for details:
- [tty-server/README.md](tty-server/README.md) - Session server
- [ttyclaude2/README.md](ttyclaude2/README.md) - Terminal client
- [tty-web/README.md](tty-web/README.md) - Web client
- [ARCHITECTURE.md](ARCHITECTURE.md) - Full technical specs

## Contributing

This is an open design process. See [DESIGN.md](DESIGN.md) for original architecture and [ARCHITECTURE.md](ARCHITECTURE.md) for ttyclaude2.

**Areas needing decisions:**
- TUI framework (tui-rs vs raw terminal)
- Message storage format
- Conversation branching UX
- Multi-line input handling

**Pull requests welcome for:**
- Core functionality
- Bug fixes
- Documentation
- Tests

**Not accepting:**
- GUI/web versions
- Telemetry/analytics
- "Enterprise" features
- Complexity without clear value

## Development

```bash
# Build
cargo build

# Run
cargo run

# Test
cargo test

# Check
cargo clippy
```

## License

TBD - Likely GPL-3.0 or MIT

## Acknowledgments

Inspired by:
- irssi, weechat - Terminal chat done right
- vim - Modal editing paradigm
- screen, tmux - Terminal multiplexing
- Claude Code - API integration for developers
- broot - Excellent use of termimad

Built as a learning project to explore Rust + async + TUI development while solving a real workflow problem.

## Contact

- Issues: <https://codeberg.org/Threadpanic/ttyclaude/issues>
- Repo: <https://codeberg.org/Threadpanic/ttyclaude>

---

Built by terminal nerds, for terminal nerds.
