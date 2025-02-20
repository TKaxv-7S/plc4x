use tokio::net::TcpStream;
use crate::spi::{Transport, TransportError};
use crate::spi::config::TcpConfig;
use std::io;

pub struct TcpTransport {
    stream: Option<TcpStream>,
    address: String,
    port: u16,
    config: TcpConfig,
}

impl TcpTransport {
    pub fn new(address: String, port: u16) -> Self {
        Self::new_with_config(address, port, TcpConfig {
            base: Default::default(),
            no_delay: true,
            keep_alive: true,
        })
    }

    pub fn new_with_config(address: String, port: u16, config: TcpConfig) -> Self {
        TcpTransport {
            stream: None,
            address,
            port,
            config,
        }
    }
}

impl Transport for TcpTransport {
    async fn connect_internal(&mut self) -> Result<(), TransportError> {
        if self.stream.is_some() {
            return Err(TransportError::AlreadyConnected);
        }
        
        let addr = format!("{}:{}", self.address, self.port);
        let stream = TcpStream::connect(addr).await?;
        
        // Apply TCP-specific settings
        stream.set_nodelay(self.config.no_delay)?;
        stream.set_keepalive(self.config.keep_alive.then_some(self.config.base.connect_timeout))?;
        
        self.stream = Some(stream);
        Ok(())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        let stream = self.stream.as_mut()
            .ok_or(TransportError::NotConnected)?;
            
        use tokio::io::AsyncReadExt;
        Ok(stream.read(buffer).await?)
    }

    async fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        let stream = self.stream.as_mut()
            .ok_or(TransportError::NotConnected)?;
            
        use tokio::io::AsyncWriteExt;
        Ok(stream.write(data).await?)
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        if let Some(stream) = self.stream.take() {
            use tokio::io::AsyncWriteExt;
            stream.shutdown().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_tcp_transport() {
        let mut transport = TcpTransport::new("127.0.0.1".to_string(), 102);
        
        // Test connection
        block_on(async {
            assert!(transport.connect().await.is_err()); // Should fail as no server is running
            assert!(transport.stream.is_none());
        });
    }
} 
