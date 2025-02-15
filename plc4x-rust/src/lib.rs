//! PLC4X Rust implementation for S7 protocol

mod error;
mod s7;
mod types;

pub use error::Error;
pub use s7::{S7Connector, S7Header, MessageType};
pub use types::{CommunicationType, FunctionCode, AreaCode}; 
