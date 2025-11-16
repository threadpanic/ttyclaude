# ttyclaude Mental Model

## The Core Insight

You have linear, story-like cognition. Threading traditionally breaks this narrative flow with shared state, locks, and memory ordering concerns. But ttyclaude doesn't need any of that.

## The Architecture: Message Passing

Think Unix philosophy: processes communicating via pipes, not shared memory.

```
+-------------------+
|   Main Thread     |  <- Your story lives here
|   (UI + Input)    |     Terminal events, rendering, keyboard
+---------+---------+
          |
+---------+---------+
|     AppState      |  <- Also lives here, your bookkeeping
| +---------------+ |
| | sessions:     | |
| | [Opus:Anthr]  | |
| | [Local:Llama] | |
| | [Quick:OAI]   | |
| +---------------+ |
+---------+---------+
          | channels (one-way pipes)
     +----+----+---------+
     v         v         v
+-------+ +-------+ +-------+
| Opus  | | Local | |Sonnet |
|Buffer | |Buffer | |Buffer |
+-------+ +-------+ +-------+
    ^         ^         ^
    HTTP streaming (isolated, you don't care)
```

## Stage 1: Basic Skeleton

Get tokio/ratatui working with message passing:
- Main loop reads keystrokes
- One dummy session echoes back
- `select!` handles both
- Prove the architecture works

## Stage 2: Provider Abstraction

Add the type system for session management:

```rust
enum ProviderType {
    Anthropic,
    OpenAICompat,  // LMStudio, OpenRouter, etc.
    LlamaCpp,
}

struct SessionInfo {
    id: SessionId,
    name: String,              // "opus/architecture"
    provider: ProviderType,
    cmd_tx: Sender<SessionCommand>,
}

struct AppState {
    sessions: HashMap<SessionId, SessionInfo>,
    active_session: SessionId,
}
```

This enables subsetting and generalization:

```rust
// All Anthropic buffers
app.sessions.values().filter(|s| s.provider == Anthropic)

// All local buffers
app.sessions.values().filter(|s| s.provider == LlamaCpp)
```

Still no shared state between threads. AppState lives entirely in your main thread narrative. Session threads receive their config at spawn time and forget about the rest of the world.

## Why This Works For Your Brain

**Main thread = your linear narrative**
- Read keystroke
- Dispatch command
- Check for responses
- Render
- Repeat

No branching storylines. No wondering "what's that other thread doing right now?" Each session is a black box that accepts commands and emits events.

**Session threads = subprocesses**
Like `fork()` + `pipe()`, except:
- `tokio::spawn()` instead of `fork()`
- `mpsc::channel()` instead of `pipe()`
- Same isolation guarantees

## The Two Message Types

```rust
// You -> Session (commands go down)
enum SessionCommand {
    SendMessage(String),
    Cancel,
    Shutdown,
}

// Session -> You (events come up)
enum SessionEvent {
    TokenReceived(String),
    ResponseComplete,
    Error(String),
}
```

That's the entire inter-thread API. No mutexes, no atomics, no memory fences.

## Your Input Rules Supreme

Main thread never blocks on network I/O. Ever.

- Keystroke? Immediate response.
- Switch buffer? Instant.
- Send message? Dispatch to channel, return immediately.
- Cancel streaming? Send `Cancel` command, done.

The HTTP request/response cycle happens in a separate universe (the session thread). You just see events trickling back: "token arrived", "token arrived", "complete".

## The irssi/weechat Parallel

You're building a chat client where:
- Providers = IRC servers
- Buffers = channels/queries  
- Streaming responses = incoming messages
- Your typing = outgoing messages

Same mental model as IRC clients you already understand. Different transport layer (HTTP SSE instead of TCP IRC protocol), identical architecture.

## What You're NOT Doing

- No shared mutable state between threads
- No lock-free data structures
- No memory ordering concerns
- No "what if thread A reads while thread B writes"
- No condition variables or semaphores

Just pipes. Messages go in, messages come out. The session thread's internal state is its own business.

## Tokio Makes This Trivial

```rust
let (cmd_tx, cmd_rx) = mpsc::channel(32);
let (evt_tx, evt_rx) = mpsc::channel(32);

// Spawn session - it's gone, doing its thing
tokio::spawn(session_loop(cmd_rx, evt_tx));

// Main loop - your linear story
loop {
    select! {
        key = read_key() => handle_input(key, &cmd_tx),
        evt = evt_rx.recv() => update_buffer(evt),
    }
}
```

The `select!` is just "check both pipes, handle whichever has data." Still linear, still a story, just with occasional checks for mail.
