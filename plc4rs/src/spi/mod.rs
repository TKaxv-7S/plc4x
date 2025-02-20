//! Service Provider Interface (SPI) for PLC4X
//! 
//! This module provides the core abstractions for implementing different transport
//! mechanisms in PLC4X. The main trait is `Transport` which defines the basic
//! operations that any transport implementation must provide.
//!
//! # Transport Types
//! 
//! Currently implemented:
//! - TCP: For TCP/IP based protocols
//!
//! # Example
//! ```rust
//! use plc4rs::spi::{Transport, TcpTransport};
//! 
//! async fn example() {
//!     let mut transport = TcpTransport::new("192.168.1.1".to_string(), 102);
//!     transport.connect().await.unwrap();
//!     // ... use transport ...
//!     transport.close().await.unwrap();
//! }
//! ```

use std::fmt::Debug;
use tracing::{debug, error, info, warn};

/// Retry configuration for transport operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Delay between retry attempts
    pub retry_delay: std::time::Duration,
    /// Whether to use exponential backoff
    pub use_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay: std::time::Duration::from_millis(100),
            use_backoff: true,
        }
    }
}

/// Core trait for implementing transport mechanisms
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Establishes a connection to the target device
    async fn connect(&mut self) -> Result<(), TransportError> {
        self.connect_with_retry(RetryConfig::default()).await
    }
    
    /// Connects with retry logic
    async fn connect_with_retry(&mut self, retry_config: RetryConfig) -> Result<(), TransportError> {
        let mut attempt = 0;
        let mut delay = retry_config.retry_delay;

        loop {
            attempt += 1;
            match self.connect_internal().await {
                Ok(()) => {
                    info!("Connection established on attempt {}", attempt);
                    return Ok(());
                }
                Err(e) => {
                    if attempt >= retry_config.max_attempts {
                        error!("Connection failed after {} attempts: {}", attempt, e);
                        return Err(e);
                    }
                    warn!("Connection attempt {} failed: {}", attempt, e);
                    tokio::time::sleep(delay).await;
                    if retry_config.use_backoff {
                        delay *= 2;
                    }
                }
            }
        }
    }

    /// Internal connect implementation
    #[doc(hidden)]
    async fn connect_internal(&mut self) -> Result<(), TransportError>;
    
    /// Reads data with logging
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        debug!("Attempting to read {} bytes", buffer.len());
        match self.read_internal(buffer).await {
            Ok(n) => {
                debug!("Successfully read {} bytes", n);
                Ok(n)
            }
            Err(e) => {
                error!("Read error: {}", e);
                Err(e)
            }
        }
    }

    /// Internal read implementation
    #[doc(hidden)]
    async fn read_internal(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError>;
    
    /// Writes data with logging
    async fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        debug!("Attempting to write {} bytes", data.len());
        match self.write_internal(data).await {
            Ok(n) => {
                debug!("Successfully wrote {} bytes", n);
                Ok(n)
            }
            Err(e) => {
                error!("Write error: {}", e);
                Err(e)
            }
        }
    }

    /// Internal write implementation
    #[doc(hidden)]
    async fn write_internal(&mut self, data: &[u8]) -> Result<usize, TransportError>;
    
    /// Closes the connection with logging
    async fn close(&mut self) -> Result<(), TransportError> {
        info!("Closing connection");
        self.close_internal().await
    }

    /// Internal close implementation
    #[doc(hidden)]
    async fn close_internal(&mut self) -> Result<(), TransportError>;
}

// Implement transport types
pub mod tcp;
pub mod error;

pub use error::TransportError;
pub use tcp::TcpTransport;
