use thiserror::Error;

#[derive(Error, Debug)]
pub enum S7Error {
    #[error("Invalid message type: {0:#04x}")]
    InvalidMessageType(u8),
    
    #[error("Invalid function code: {0:#04x}")]
    InvalidFunctionCode(u8),
    
    #[error("Invalid parameter type: {0:#04x}")]
    InvalidParameterType(u8),
    
    #[error("Invalid protocol ID: expected 0x32, got {0:#04x}")]
    InvalidProtocolId(u8),
    
    #[error("Invalid TPKT version: expected 0x03, got {0:#04x}")]
    InvalidTpktVersion(u8),
    
    #[error("Invalid length: {0}")]
    InvalidLength(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
} 
