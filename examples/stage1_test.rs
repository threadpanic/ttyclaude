use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
};

#[tokio::main]
async fn main() {
    println!("=== Stage 1: Library Verification ===\n");

    // Test tokio async runtime
    println!("Testing tokio async runtime...");
    let result = tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        "Async task completed"
    })
    .await
    .unwrap();
    println!("  ✓ {}", result);

    // Test multiple concurrent tasks
    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(10 * i)).await;
                format!("Task {} done", i)
            })
        })
        .collect();

    for handle in handles {
        println!("  ✓ {}", handle.await.unwrap());
    }

    // Test ratatui layout without terminal
    println!("\nTesting ratatui layout engine...");
    let area = Rect::new(0, 0, 80, 24);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    println!("  ✓ Layout split into {} chunks", chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        println!("    Chunk {}: {}x{} at ({},{})", i, chunk.width, chunk.height, chunk.x, chunk.y);
    }

    // Test widget creation
    let _block = Block::default()
        .borders(Borders::ALL)
        .title("ttyclaude");
    println!("  ✓ Widget creation successful");

    println!("\n=== Stage 1 Complete ===");
    println!("✓ tokio: Async runtime working");
    println!("✓ ratatui: Layout and widgets working");
    println!("\nBoth libraries loaded and functional!");
}
