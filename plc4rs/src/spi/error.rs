use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Not connected")]
    NotConnected,
    
    #[error("Already connected")]
    AlreadyConnected,
} 
