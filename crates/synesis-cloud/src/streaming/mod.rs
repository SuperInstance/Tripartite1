//! Response streaming
//!
//! Server-sent events streaming for real-time responses

use crate::error::CloudResult;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Streamed response chunk
///
/// Represents a single chunk in a streamed response.
/// Chunks are delivered sequentially and can be processed
/// as they arrive for real-time display.
///
/// # Fields
///
/// - `content`: The text content for this chunk
/// - `sequence`: Monotonically increasing sequence number (0-indexed)
/// - `is_final`: True if this is the last chunk
///
/// # Example
///
/// ```rust,no_run
/// use synesis_cloud::streaming::StreamChunk;
///
/// let chunk = StreamChunk {
///     content: "Hello".to_string(),
///     sequence: 0,
///     is_final: false,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Text content for this chunk
    pub content: String,
    /// Sequence number (monotonically increasing)
    pub sequence: u32,
    /// True if this is the final chunk
    pub is_final: bool,
}

/// Streaming response receiver
///
/// Receives chunks from a streaming escalation response.
/// Chunks can be processed individually or collected into a complete response.
///
/// # Thread Safety
///
/// This struct is not thread-safe and should not be shared.
/// Create a new receiver for each concurrent stream.
///
/// # Example
///
/// ```rust,no_run
/// use synesis_cloud::streaming::StreamingResponse;
/// # use tokio::sync::mpsc;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// # let (tx, rx) = mpsc::channel(100);
/// # let _ = tx; // sender
/// let mut stream = StreamingResponse::new(rx);
///
/// // Process chunks as they arrive
/// while let Some(chunk) = stream.recv_chunk().await? {
///     print!("{}", chunk.content);
///     if chunk.is_final {
///         break;
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub struct StreamingResponse {
    receiver: mpsc::Receiver<StreamChunk>,
}

impl StreamingResponse {
    /// Create new streaming response
    pub fn new(receiver: mpsc::Receiver<StreamChunk>) -> Self {
        Self { receiver }
    }

    /// Receive next chunk
    pub async fn recv_chunk(&mut self) -> CloudResult<Option<StreamChunk>> {
        Ok(self.receiver.recv().await)
    }

    /// Collect all chunks into final string
    pub async fn collect(mut self) -> CloudResult<String> {
        let mut content = String::new();

        while let Some(chunk) = self.recv_chunk().await? {
            content.push_str(&chunk.content);

            if chunk.is_final {
                break;
            }
        }

        Ok(content)
    }
}

/// Stream builder for escalation requests
///
/// Builder for starting streaming escalation requests via QUIC tunnel.
///
/// # Production Status
///
/// The `tunnel` field is reserved for future QUIC tunnel integration.
pub struct StreamBuilder {
    #[allow(dead_code)]
    tunnel: Arc<crate::tunnel::tunnel::CloudTunnel>,
}

impl StreamBuilder {
    /// Create new stream builder
    pub fn new(tunnel: Arc<crate::tunnel::tunnel::CloudTunnel>) -> Self {
        Self { tunnel }
    }

    /// Start streaming escalation
    pub async fn escalate(&self, _request: crate::escalation::types::EscalationRequest) -> CloudResult<StreamingResponse> {
        // TODO: Implement actual streaming via QUIC
        // For now, return a mock stream
        let (tx, rx) = mpsc::channel(100);

        // Mock: send a single chunk
        let _ = tx.send(StreamChunk {
            content: "Mock streaming response".to_string(),
            sequence: 0,
            is_final: true,
        }).await;

        Ok(StreamingResponse::new(rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stream_collect() {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let _ = tx.send(StreamChunk {
                content: "Hello".to_string(),
                sequence: 0,
                is_final: false,
            }).await;

            let _ = tx.send(StreamChunk {
                content: " World".to_string(),
                sequence: 1,
                is_final: true,
            }).await;
        });

        let stream = StreamingResponse::new(rx);
        let collected = stream.collect().await.unwrap();

        assert_eq!(collected, "Hello World");
    }

    #[tokio::test]
    async fn test_stream_recv_chunk() {
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let _ = tx.send(StreamChunk {
                content: "Test".to_string(),
                sequence: 0,
                is_final: true,
            }).await;
        });

        let mut stream = StreamingResponse::new(rx);

        let chunk1 = stream.recv_chunk().await.unwrap().unwrap();
        assert_eq!(chunk1.content, "Test");
        assert!(chunk1.is_final);

        let chunk2 = stream.recv_chunk().await.unwrap();
        assert!(chunk2.is_none());
    }
}
