use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub connect_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
    pub buffer_size: usize,
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(1),
            write_timeout: Duration::from_secs(1),
            buffer_size: 8192,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TcpConfig {
    pub base: TransportConfig,
    pub no_delay: bool,
    pub keep_alive: bool,
}
