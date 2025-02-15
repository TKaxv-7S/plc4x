//! S7 protocol type definitions

/// S7 Communication Type
#[derive(Debug, Clone, Copy)]
pub enum CommunicationType {
    PG = 0x01,
    OP = 0x02,
    Step7Basic = 0x03,
}

/// S7 Function Code
#[derive(Debug, Clone, Copy)]
pub enum FunctionCode {
    ReadVar = 0x04,
    WriteVar = 0x05,
    RequestDownload = 0x1A,
    Download = 0x1B,
    DownloadEnded = 0x1C,
    // Add more as needed
}

/// S7 Area Code
#[derive(Debug, Clone, Copy)]
pub enum AreaCode {
    SysInfo = 0x03,
    SysFlags = 0x05,
    AnaInput = 0x06,
    AnaOutput = 0x07,
    P = 0x80,
    Inputs = 0x81,
    Outputs = 0x82,
    Flags = 0x83,
    DB = 0x84,
    DI = 0x85,
    Local = 0x86,
    V = 0x87,
} 
