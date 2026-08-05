#[derive(Debug, Clone)]
pub struct KrpcConfig {
    pub client_name: String,
    pub address: String,
    pub rpc_port: u16,

    pub battery_hz: f32,
    pub navigation_hz: f32,
    pub attitude_hz: f32,
    pub propulsion_hz: f32,
}

impl Default for KrpcConfig {
    fn default() -> Self {
        Self {
            client_name: "KTYTOI SPYTNIK NAXYI".to_owned(),
            address: "127.0.0.1".to_owned(),
            rpc_port: 50_000,

            battery_hz: 2.0,
            navigation_hz: 10.0,
            attitude_hz: 20.0,
            propulsion_hz: 10.0,
        }
    }
}