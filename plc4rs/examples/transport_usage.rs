use plc4rs::spi::{
    TcpTransport,
    config::{TransportConfig, TcpConfig},
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TCP Example
    let tcp_config = TcpConfig {
        base: TransportConfig {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(2),
            write_timeout: Duration::from_secs(2),
            buffer_size: 1024,
        },
        no_delay: true,
        keep_alive: true,
    };

    let mut tcp = TcpTransport::new_with_config("192.168.1.1".into(), 102, tcp_config);
    tcp.connect().await?;
    
    let data = b"Hello PLC";
    tcp.write(data).await?;
    
    let mut buffer = vec![0u8; 1024];
    let len = tcp.read(&mut buffer).await?;
    println!("Received: {:?}", &buffer[..len]);
    
    tcp.close().await?;
    
    Ok(())
} 
