use anyhow::{Result, anyhow};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use bytes::{BytesMut, BufMut};
use tracing::{debug, error};

use super::messages::{ClientMessage, ServerMessage};

pub struct Client {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Client {
    pub async fn connect(addr: &str) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        debug!("Connected to server at {}", addr);

        Ok(Self {
            stream,
            buffer: BytesMut::with_capacity(4096),
        })
    }

    pub async fn send(&mut self, msg: &ClientMessage) -> Result<()> {
        let json = serde_json::to_vec(msg)?;
        let len = json.len() as u32;

        let mut frame = BytesMut::with_capacity(4 + json.len());
        frame.put_u32(len);
        frame.put_slice(&json);

        self.stream.write_all(&frame).await?;
        debug!("Sent message: {:?}", msg);

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<ServerMessage> {
        // Read frame length
        let mut len_bytes = [0u8; 4];
        self.stream.read_exact(&mut len_bytes).await?;
        let frame_len = u32::from_be_bytes(len_bytes) as usize;

        if frame_len > 10_000_000 {
            return Err(anyhow!("Frame too large: {} bytes", frame_len));
        }

        // Read frame data
        self.buffer.clear();
        self.buffer.resize(frame_len, 0);
        self.stream.read_exact(&mut self.buffer).await?;

        // Parse message
        let msg: ServerMessage = serde_json::from_slice(&self.buffer)?;
        debug!("Received message: {:?}", msg);

        Ok(msg)
    }

    pub async fn try_recv(&mut self) -> Result<Option<ServerMessage>> {
        // Check if we can read without blocking
        self.stream.readable().await?;

        // Try to peek at the length
        let mut len_bytes = [0u8; 4];
        match self.stream.try_read(&mut len_bytes) {
            Ok(n) if n == 4 => {
                let frame_len = u32::from_be_bytes(len_bytes) as usize;

                if frame_len > 10_000_000 {
                    return Err(anyhow!("Frame too large: {} bytes", frame_len));
                }

                // Read the rest
                self.buffer.clear();
                self.buffer.resize(frame_len, 0);
                self.stream.read_exact(&mut self.buffer).await?;

                let msg: ServerMessage = serde_json::from_slice(&self.buffer)?;
                debug!("Received message: {:?}", msg);

                Ok(Some(msg))
            }
            Ok(_) => {
                // Partial read, wait for more
                Ok(None)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }
}
