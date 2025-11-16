# ttyclaude2 Architecture

## Overview

ttyclaude2 is a distributed architecture for LLM conversations with multiple client types:

```
┌──────────────┐         ┌─────────────────────────┐         ┌──────────────┐
│  ttyclaude2  │◀───────▶│   Session Server        │◀───────▶│  Web Client  │
│  (terminal)  │  native │   (tty-server)          │  HTTP/WS│  (React/TSX) │
└──────────────┘  proto  │                         │         └──────────────┘
                         │  ┌──────────────────┐   │
                         │  │ Core             │   │
                         │  │ - Sessions       │   │
                         │  │ - SQLite         │   │
                         │  │ - Auth (JWT)     │   │
                         │  │ - Providers      │   │
                         │  └──────────────────┘   │
                         └─────────────────────────┘
                                    │
                                    ▼
                         ┌─────────────────────────┐
                         │   LLM Providers         │
                         │  - Anthropic            │
                         │  - OpenAI               │
                         │  - Local (llama.cpp)    │
                         └─────────────────────────┘
```

## Components

### 1. Session Server (`tty-server`)

**Purpose**: Central session coordination, persistence, and provider routing

**Listeners**:
- Native Protocol: `:7331` (TCP) or `/run/ttyclaude/session.sock` (Unix)
- HTTP/WebSocket: `:7332`

**Core Responsibilities**:
- Session lifecycle management (create, resume, list, delete)
- Message persistence (SQLite)
- Authentication & authorization (JWT)
- Provider abstraction & routing
- Token streaming coordination

**Tech Stack**:
- Rust + Tokio
- axum (HTTP server)
- tokio-tungstenite (WebSocket)
- rusqlite (database)
- jsonwebtoken (auth)

### 2. ttyclaude2 (Terminal Client)

**Purpose**: Fast, keyboard-driven terminal interface

**Features**:
- Vim-style keybindings
- Streaming markdown rendering (termimad + syntect)
- Sub-100ms startup
- Offline queue support

**Protocol**: Native binary/JSON over TCP/Unix socket

**Tech Stack**:
- Rust + Tokio
- ratatui + crossterm (TUI)
- termimad (markdown)
- syntect (syntax highlighting)

### 3. Web Client (`tty-web`)

**Purpose**: Browser-based access for non-terminal contexts

**Features**:
- React-based UI
- WebSocket streaming
- Markdown rendering (react-markdown)
- Code highlighting (highlight.js)

**Tech Stack**:
- TypeScript + React
- Vite (build tool)
- TanStack Query (data fetching)
- Tailwind CSS

## Protocol Specifications

### Native Protocol (ttyclaude2 ↔ server)

**Transport**: TCP or Unix socket, length-prefixed JSON frames

**Frame Format**:
```
[4 bytes: length (u32 big-endian)][JSON payload]
```

**Messages (Client → Server)**:
```json
{
  "type": "authenticate",
  "token": "jwt_token_here"
}

{
  "type": "create_session",
  "provider": "anthropic",
  "model": "claude-sonnet-4-20250514"
}

{
  "type": "send_message",
  "session_id": "uuid",
  "content": "Hello, Claude!"
}

{
  "type": "list_sessions"
}

{
  "type": "load_session",
  "session_id": "uuid"
}
```

**Messages (Server → Client)**:
```json
{
  "type": "authenticated",
  "user_id": "uuid"
}

{
  "type": "session_created",
  "session_id": "uuid"
}

{
  "type": "token",
  "session_id": "uuid",
  "content": "Hello"
}

{
  "type": "message_complete",
  "session_id": "uuid",
  "message_id": "uuid"
}

{
  "type": "error",
  "code": "INVALID_TOKEN",
  "message": "Authentication failed"
}

{
  "type": "sessions_list",
  "sessions": [{"id": "uuid", "created_at": "...", "preview": "..."}]
}
```

### HTTP/WebSocket API (Web client ↔ server)

**REST Endpoints**:
```
POST   /api/v1/auth/login         # Get JWT token
GET    /api/v1/sessions           # List sessions
POST   /api/v1/sessions           # Create session
GET    /api/v1/sessions/:id       # Get session details
DELETE /api/v1/sessions/:id       # Delete session
GET    /api/v1/sessions/:id/messages  # Get message history
```

**WebSocket** (`/ws`):
```json
// Client → Server
{
  "type": "authenticate",
  "token": "jwt_token"
}

{
  "type": "send_message",
  "session_id": "uuid",
  "content": "Hello!"
}

// Server → Client (streaming)
{
  "type": "token",
  "session_id": "uuid",
  "content": "Hi"
}

{
  "type": "message_complete",
  "session_id": "uuid"
}
```

## Database Schema

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant'
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY(session_id) REFERENCES sessions(id)
);

CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_messages_session ON messages(session_id, created_at);
CREATE INDEX idx_sessions_user ON sessions(user_id, updated_at DESC);
```

## Authentication Flow

1. User provides credentials (username/password or API key)
2. Server validates and issues JWT (24hr expiry)
3. Client includes JWT in all requests
4. JWT contains: `user_id`, `exp`, `iat`
5. Server validates signature on every request

## Configuration

**Server** (`~/.config/tty-server/config.toml`):
```toml
[server]
native_bind = "127.0.0.1:7331"  # or unix socket path
http_bind = "127.0.0.1:7332"
database_path = "~/.local/share/tty-server/sessions.db"

[auth]
jwt_secret = "generate-with-openssl-rand"
token_expiry_hours = 24

[providers.anthropic]
api_key = "sk-ant-..."
default_model = "claude-sonnet-4-20250514"

[providers.openai]
api_key = "sk-..."
default_model = "gpt-4"

[providers.local]
type = "openai-compatible"
endpoint = "http://localhost:8080"
default_model = "phi-4"
```

**Client** (`~/.config/ttyclaude2/config.toml`):
```toml
[server]
address = "127.0.0.1:7331"  # or unix socket path

[auth]
# Auto-generated on first run
token = "jwt_token_here"

[ui]
vim_mode = true
theme = "monokai"
```

## Deployment Scenarios

### Local Only
- Server: `127.0.0.1` or Unix socket
- Clients connect locally
- No TLS needed

### VPS Deployment
- Server: VPS with public IP
- Native: SSH tunnel (`ssh -L 7331:localhost:7331 vps`)
- HTTP: TLS reverse proxy (nginx/caddy)
- Firewall: Only expose :443 (HTTPS)

### Self-Hosted Team
- Server: Internal network
- mTLS for native protocol
- OAuth2 for web client

## Development Roadmap

1. **v0.1** - Session server core + native protocol
2. **v0.2** - ttyclaude2 basic client (connect, send, stream)
3. **v0.3** - Web client prototype
4. **v0.4** - Full feature parity (search, export, etc.)
5. **v0.5** - Multi-user, team features
