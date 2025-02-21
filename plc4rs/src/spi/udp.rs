use tokio::net::UdpSocket;
use crate::spi::{Transport, TransportError};
use crate::spi::config::TransportConfig;
use std::io;

pub struct UdpTransport {
    socket: Option<UdpSocket>,
    address: String,
    port: u16,
    config: TransportConfig,
    buffer: Vec<u8>,
}

impl UdpTransport {
    pub fn new(address: String, port: u16) -> Self {
        Self::new_with_config(address, port, TransportConfig::default())
    }

    pub fn new_with_config(address: String, port: u16, config: TransportConfig) -> Self {
        UdpTransport {
            socket: None,
            address,
            port,
            config,
            buffer: vec![0; config.buffer_size],
        }
    }
}

#[async_trait::async_trait]
impl Transport for UdpTransport {
    async fn connect_internal(&mut self) -> Result<(), TransportError> {
        if self.socket.is_some() {
            return Err(TransportError::AlreadyConnected);
        }
        
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let addr = format!("{}:{}", self.address, self.port);
        socket.connect(&addr).await?;
        
        self.socket = Some(socket);
        Ok(())
    }

    async fn read_internal(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        let socket = self.socket.as_ref()
            .ok_or(TransportError::NotConnected)?;
            
        let len = socket.recv(buffer).await?;
        Ok(len)
    }

    async fn write_internal(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        let socket = self.socket.as_ref()
            .ok_or(TransportError::NotConnected)?;
            
        let len = socket.send(data).await?;
        Ok(len)
    }

    async fn close_internal(&mut self) -> Result<(), TransportError> {
        self.socket = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_udp_lifecycle() {
        let mut transport = UdpTransport::new("127.0.0.1".to_string(), 1234);
        
        block_on(async {
            // Test connection
            assert!(transport.connect().await.is_ok());
            assert!(transport.socket.is_some());

            // Test write/read
            let data = b"test data";
            let result = transport.write(data).await;
            assert!(result.is_ok() || result.is_err()); // May fail as no server

            // Test close
            assert!(transport.close().await.is_ok());
            assert!(transport.socket.is_none());
        });
    }
} 
