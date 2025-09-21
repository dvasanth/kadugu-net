use libp2p::StreamProtocol;

#[derive(Debug, Clone)]
pub struct Config {
    pub proxy_protocol: StreamProtocol,
    pub proxy_agent: String,
    pub relay_address: String,
    pub proxy_listen_addr: String,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proxy_protocol: StreamProtocol::new("/portforward/0.1.0"),
            proxy_agent: "libp2p-port-forward".to_string(),
            relay_address: "/ip4/104.131.131.82/udp/4001/quic-v1/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ".to_string(), // Default relay address
            proxy_listen_addr: "127.0.0.1:0".to_string(), // Default listen address
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Override with environment variables if they exist
        if let Ok(relay) = std::env::var("RELAY_ADDRESS") {
            config.relay_address = relay;
        }

        config
    }
}
