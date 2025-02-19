use nom::{
    bytes::complete::take,
    number::complete::{be_u8, be_u16, be_u32},
    IResult,
};

#[derive(Debug, Clone)]
pub struct TPKTPacket {
    protocol_id: u8,    // Always 0x03
    reserved: u8,       // Always 0x00 
    length: u16,        // Total length including header
    payload: COTPPacket,
}

impl TPKTPacket {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, protocol_id) = be_u8(input)?;
        let (input, reserved) = be_u8(input)?;
        let (input, length) = be_u16(input)?;
        let (input, payload) = COTPPacket::parse(input)?;

        Ok((input, TPKTPacket {
            protocol_id,
            reserved,
            length,
            payload,
        }))
    }
}

#[derive(Debug, Clone)]
pub enum COTPPacket {
    Data(COTPDataPacket),
    ConnectionRequest(COTPConnectionRequest),
    ConnectionResponse(COTPConnectionResponse),
}

impl COTPPacket {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header_length) = be_u8(input)?;
        let (input, tpdu_code) = be_u8(input)?;
        
        match tpdu_code {
            0xF0 => {
                let (input, packet) = COTPDataPacket::parse(input)?;
                Ok((input, COTPPacket::Data(packet)))
            },
            0xE0 => {
                let (input, packet) = COTPConnectionRequest::parse(input)?;
                Ok((input, COTPPacket::ConnectionRequest(packet))) 
            },
            0xD0 => {
                let (input, packet) = COTPConnectionResponse::parse(input)?;
                Ok((input, COTPPacket::ConnectionResponse(packet)))
            },
            _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct COTPDataPacket {
    eot: bool,      // End of transmission
    tpdu_ref: u8,   // Reference number
    data: S7Message, // S7 protocol data
}

impl COTPDataPacket {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, flags) = be_u8(input)?;
        let (input, tpdu_ref) = be_u8(input)?;
        let (input, data) = S7Message::parse(input)?;

        Ok((input, COTPDataPacket {
            eot: (flags & 0x80) != 0,
            tpdu_ref,
            data,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct S7Message {
    header: S7Header,
    parameters: Option<S7Parameters>,
    payload: Option<S7Payload>,
}

impl S7Message {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, header) = S7Header::parse(input)?;
        let (input, parameters) = if header.parameter_length > 0 {
            let (input, params) = S7Parameters::parse(input, header.parameter_length)?;
            (input, Some(params))
        } else {
            (input, None)
        };
        let (input, payload) = if header.data_length > 0 {
            let (input, data) = S7Payload::parse(input, header.data_length)?;
            (input, Some(data))
        } else {
            (input, None)
        };

        Ok((input, S7Message {
            header,
            parameters,
            payload,
        }))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MessageType {
    JobRequest = 0x01,
    Ack = 0x02,
    AckData = 0x03,
    UserData = 0x07,
}

impl TryFrom<u8> for MessageType {
    type Error = S7Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MessageType::JobRequest),
            0x02 => Ok(MessageType::Ack),
            0x03 => Ok(MessageType::AckData),
            0x07 => Ok(MessageType::UserData),
            _ => Err(S7Error::InvalidMessageType(value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct S7Header {
    protocol_id: u8,         // Always 0x32
    message_type: MessageType,
    reserved: u16,           // Always 0x0000
    pdu_reference: u16,
    parameter_length: u16,
    data_length: u16,
}

impl S7Header {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, protocol_id) = be_u8(input)?;
        let (input, msg_type) = be_u8(input)?;
        let (input, reserved) = be_u16(input)?;
        let (input, pdu_reference) = be_u16(input)?;
        let (input, parameter_length) = be_u16(input)?;
        let (input, data_length) = be_u16(input)?;

        let message_type = MessageType::try_from(msg_type)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            )))?;

        Ok((input, S7Header {
            protocol_id,
            message_type,
            reserved,
            pdu_reference,
            parameter_length,
            data_length,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct S7Parameters {
    parameter_type: ParameterType,
    items: Vec<ParameterItem>,
}

#[derive(Debug, Clone, Copy)]
pub enum ParameterType {
    Setup = 0xF0,
    ReadVar = 0x04,
    WriteVar = 0x05,
    StartUpload = 0x1D,
    Upload = 0x1E,
    EndUpload = 0x1F,
    // Add more as needed
}

impl TryFrom<u8> for ParameterType {
    type Error = S7Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xF0 => Ok(ParameterType::Setup),
            0x04 => Ok(ParameterType::ReadVar),
            0x05 => Ok(ParameterType::WriteVar),
            0x1D => Ok(ParameterType::StartUpload),
            0x1E => Ok(ParameterType::Upload),
            0x1F => Ok(ParameterType::EndUpload),
            _ => Err(S7Error::InvalidParameterType(value)),
        }
    }
}

impl S7Parameters {
    pub fn parse(input: &[u8], length: u16) -> IResult<&[u8], Self> {
        let (input, param_type) = be_u8(input)?;
        let parameter_type = ParameterType::try_from(param_type)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            )))?;

        let (input, items) = match parameter_type {
            ParameterType::ReadVar | ParameterType::WriteVar => {
                let (input, item_count) = be_u8(input)?;
                let mut items = Vec::with_capacity(item_count as usize);
                let mut remaining = input;
                
                for _ in 0..item_count {
                    let (input, item) = ParameterItem::parse(remaining)?;
                    items.push(item);
                    remaining = input;
                }
                (remaining, items)
            },
            _ => (input, Vec::new()),
        };

        Ok((input, S7Parameters {
            parameter_type,
            items,
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ParameterItem {
    var_spec: VarSpec,
    addr_length: u8,
    syntax_id: SyntaxId,
    area: Area,
    db_number: u16,
    start_addr: u32,
    length: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum VarSpec {
    Bit = 0x01,
    Byte = 0x02,
    Word = 0x04,
    DWord = 0x06,
    // Add more as needed
}

impl TryFrom<u8> for VarSpec {
    type Error = S7Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(VarSpec::Bit),
            0x02 => Ok(VarSpec::Byte),
            0x04 => Ok(VarSpec::Word),
            0x06 => Ok(VarSpec::DWord),
            _ => Err(S7Error::InvalidVarSpec(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SyntaxId {
    S7Any = 0x10,
    // Add more as needed
}

impl TryFrom<u8> for SyntaxId {
    type Error = S7Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(SyntaxId::S7Any),
            _ => Err(S7Error::InvalidSyntaxId(value)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Area {
    Inputs = 0x81,
    Outputs = 0x82,
    Flags = 0x83,
    DataBlocks = 0x84,
    // Add more as needed
}

impl TryFrom<u8> for Area {
    type Error = S7Error;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x81 => Ok(Area::Inputs),
            0x82 => Ok(Area::Outputs),
            0x83 => Ok(Area::Flags),
            0x84 => Ok(Area::DataBlocks),
            _ => Err(S7Error::InvalidArea(value)),
        }
    }
}

impl ParameterItem {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, var_spec) = be_u8(input)?;
        let (input, length) = be_u8(input)?;
        let (input, syntax_id) = be_u8(input)?;
        let (input, transport_size) = be_u8(input)?;
        let (input, db_number) = be_u16(input)?;
        let (input, area) = be_u8(input)?;
        let (input, start_addr) = be_u32(input)?;

        // ... implement conversion and validation ...
        
        Ok((input, ParameterItem {
            var_spec: VarSpec::Byte, // TODO: proper conversion
            addr_length: length,
            syntax_id: SyntaxId::S7Any, // TODO: proper conversion
            area: Area::DataBlocks, // TODO: proper conversion
            db_number,
            start_addr,
            length: 0, // TODO: calculate from transport_size
        }))
    }
}

// ... more implementations following s7.mspec 

#[derive(Debug, Clone)]
pub struct S7Payload {
    items: Vec<PayloadItem>,
}

#[derive(Debug, Clone)]
pub struct PayloadItem {
    return_code: ReturnCode,
    transport_size: TransportSize,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum ReturnCode {
    Success = 0x00,
    HardwareError = 0x01,
    AccessError = 0x03,
    OutOfRange = 0x05,
    NotSupported = 0x06,
    // Add more as needed
}

#[derive(Debug, Clone, Copy)]
pub enum TransportSize {
    Bit = 0x01,
    Byte = 0x02,
    Word = 0x04,
    DWord = 0x06,
    // Add more as needed
}

impl S7Payload {
    pub fn parse(input: &[u8], length: u16) -> IResult<&[u8], Self> {
        let mut items = Vec::new();
        let mut remaining = input;
        let end_pos = length as usize;
        
        while remaining.len() > 0 && (input.len() - remaining.len()) < end_pos {
            let (input, item) = PayloadItem::parse(remaining)?;
            items.push(item);
            remaining = input;
        }
        
        Ok((remaining, S7Payload { items }))
    }
}

impl PayloadItem {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, return_code) = be_u8(input)?;
        let (input, transport_size) = be_u8(input)?;
        let (input, length) = be_u16(input)?;
        let (input, data) = take(length as usize)(input)?;
        
        Ok((input, PayloadItem {
            return_code: ReturnCode::try_from(return_code)
                .map_err(|_| nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag
                )))?,
            transport_size: TransportSize::try_from(transport_size)
                .map_err(|_| nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag
                )))?,
            data: data.to_vec(),
        }))
    }
}
