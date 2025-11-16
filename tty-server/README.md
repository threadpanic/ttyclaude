# tty-server - Session Server

Backend service for ttyclaude2 ecosystem. Manages sessions, stores conversations, and routes to LLM providers.

## Features

- **Dual Protocol Support**
  - Native binary protocol (TCP/Unix socket) for terminal clients
  - HTTP REST + WebSocket for web clients

- **Session Management**
  - Create/resume/delete conversations
  - SQLite persistence
  - Session history and search

- **Multi-Provider Support**
  - Anthropic Claude
  - OpenAI GPT
  - OpenAI-compatible APIs (llama.cpp, ollama, etc.)
  - Easy to add new providers

- **Streaming**
  - Server-Sent Events (SSE) from providers
  - Real-time token streaming to clients

- **Authentication** (planned)
  - JWT-based auth
  - Multi-user support
  - API key management

## Installation

```bash
cargo build --release
```

## Configuration

Create `~/.config/tty-server/config.toml`:

```toml
[server]
native_bind = "127.0.0.1:7331"
http_bind = "127.0.0.1:7332"
database_path = "~/.local/share/tty-server/sessions.db"

[auth]
jwt_secret = "CHANGE_ME_IN_PRODUCTION"
token_expiry_hours = 24

[providers.anthropic]
type = "anthropic"
api_key = "sk-ant-..."  # or set $ANTHROPIC_API_KEY
default_model = "claude-sonnet-4-20250514"

[providers.openai]
type = "openai"
api_key = "sk-..."
default_model = "gpt-4"

[providers.local-llama]
type = "openai-compatible"
endpoint = "http://localhost:8080"
default_model = "phi-4"
```

## Running

```bash
# Development
cargo run

# Production
cargo run --release

# With custom config
RUST_LOG=debug cargo run
```

## API Documentation

### Native Protocol (Port 7331)

Binary protocol with length-prefixed JSON frames.

**Frame Format**:
```
[4 bytes: length (u32 big-endian)][JSON payload]
```

**Messages**: See [ARCHITECTURE.md](../ARCHITECTURE.md#native-protocol)

### HTTP API (Port 7332)

**Endpoints**:
```
POST   /api/v1/sessions           - Create session
GET    /api/v1/sessions           - List sessions
GET    /api/v1/sessions/:id       - Get session
DELETE /api/v1/sessions/:id       - Delete session
GET    /api/v1/sessions/:id/messages - Get messages
```

**WebSocket** (`/ws`):
- Real-time bidirectional communication
- Token streaming
- See [ARCHITECTURE.md](../ARCHITECTURE.md#http-websocket-api)

## Database Schema

```sql
-- Users
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

-- Sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- Messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant'
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id)
);
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt

# Watch mode (requires cargo-watch)
cargo watch -x run
```

## Architecture

```
src/
├── main.rs          - Entry point, server startup
├── config.rs        - Configuration loading
├── db/
│   ├── mod.rs       - Database interface
│   └── models.rs    - Data models
├── session.rs       - Session management
├── providers/
│   └── mod.rs       - LLM provider implementations
├── protocol/
│   ├── server.rs    - Native protocol server
│   └── messages.rs  - Protocol message types
└── api/
    ├── server.rs    - HTTP/WebSocket server
    └── handlers.rs  - API route handlers
```

## Deployment

### Systemd Service

Create `/etc/systemd/system/tty-server.service`:

```ini
[Unit]
Description=tty-server - LLM Session Server
After=network.target

[Service]
Type=simple
User=ttyclaude
WorkingDirectory=/home/ttyclaude
ExecStart=/usr/local/bin/tty-server
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable tty-server
sudo systemctl start tty-server
```

### Docker (Planned)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/tty-server /usr/local/bin/
EXPOSE 7331 7332
CMD ["tty-server"]
```

## Security Notes

- Change `jwt_secret` in production
- Use TLS for remote deployments
- Store API keys in environment variables, not config files
- Implement rate limiting for production (planned)
- Use firewall rules to restrict access

## License

See [LICENSE](../LICENSE)
