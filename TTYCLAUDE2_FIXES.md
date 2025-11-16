# ttyclaude2 Compilation Fixes

## Issue
The code was written for an older ratatui API. In ratatui 0.25.0, `Frame` no longer takes a generic `Backend` parameter.

## Fixes Required

### 1. src/ui/mod.rs

**Remove unused imports (lines 7-8):**

```rust
// BEFORE:
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},  // <- Remove 'Text'
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},  // <- Remove 'List, ListItem'
    Frame,
};

// AFTER:
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
```

**Fix render method (line 212):**

```rust
// BEFORE:
pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>) {

// AFTER:
pub fn render(&mut self, frame: &mut Frame) {
```

**Fix render_messages method (line 232):**

```rust
// BEFORE:
fn render_messages<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {

// AFTER:
fn render_messages(&self, frame: &mut Frame, area: Rect) {
```

**Fix render_input method (line 259):**

```rust
// BEFORE:
fn render_input<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {

// AFTER:
fn render_input(&self, frame: &mut Frame, area: Rect) {
```

**Fix render_status method (line 266):**

```rust
// BEFORE:
fn render_status<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {

// AFTER:
fn render_status(&self, frame: &mut Frame, area: Rect) {
```

**Fix method calls (line 223 and similar):**

The compiler errors about type annotations will be resolved automatically once you remove the `<B: Backend>` generic parameters from the method signatures above.

### 2. src/protocol/client.rs

**Remove unused import (line 5):**

```rust
// BEFORE:
use tracing::{debug, error};

// AFTER:
use tracing::debug;
```

### 3. src/protocol/mod.rs

**Remove unused import (line 5):**

```rust
// BEFORE:
pub use messages::{ClientMessage, ServerMessage, SessionInfo, MessageInfo};

// AFTER:
pub use messages::{ClientMessage, ServerMessage, SessionInfo};
```

## Quick Fix Script

You can apply these fixes with sed commands:

```bash
cd ~/src/github/ttyclaude/ttyclaude2

# Fix src/ui/mod.rs
sed -i 's/text::{Line, Span, Text}/text::{Line, Span}/' src/ui/mod.rs
sed -i 's/widgets::{Block, Borders, List, ListItem, Paragraph, Wrap}/widgets::{Block, Borders, Paragraph, Wrap}/' src/ui/mod.rs
sed -i 's/pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>)/pub fn render(\&mut self, frame: \&mut Frame)/' src/ui/mod.rs
sed -i 's/fn render_messages<B: Backend>(&self, frame: &mut Frame<B>,/fn render_messages(\&self, frame: \&mut Frame,/' src/ui/mod.rs
sed -i 's/fn render_input<B: Backend>(&self, frame: &mut Frame<B>,/fn render_input(\&self, frame: \&mut Frame,/' src/ui/mod.rs
sed -i 's/fn render_status<B: Backend>(&self, frame: &mut Frame<B>,/fn render_status(\&self, frame: \&mut Frame,/' src/ui/mod.rs

# Fix src/protocol/client.rs
sed -i 's/use tracing::{debug, error};/use tracing::debug;/' src/protocol/client.rs

# Fix src/protocol/mod.rs
sed -i 's/, MessageInfo//' src/protocol/mod.rs

# Test the build
cargo build
```

## Alternative: Manual Search & Replace

If you prefer to do it manually in your editor:

1. Open `src/ui/mod.rs`
   - Find: `<B: Backend>` - Replace with: `` (empty)
   - Find: `Frame<B>` - Replace with: `Frame`
   - Remove unused imports: `Text`, `List`, `ListItem`

2. Open `src/protocol/client.rs`
   - Remove `, error` from the tracing import

3. Open `src/protocol/mod.rs`
   - Remove `, MessageInfo` from the pub use statement

## Explanation

The ratatui 0.25.0 API simplified the Frame type to no longer require backend generics. This is a breaking change from earlier versions where Frame<B: Backend> was used. The modern API is cleaner and doesn't require the generic parameter.

After applying these fixes, run:
```bash
cargo build
cargo clippy
cargo test
```
