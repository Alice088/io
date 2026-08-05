use anyhow::Error;
use stayputnik::{
    Client,
    ClientRef,
};

pub const RESOURCE_EC: &str = "ElectricCharge";
pub const RESOURCE_LIQUID_FUEL: &str = "LiquidFuel";
pub const RESOURCE_OXIDIZER: &str = "Oxidizer";
pub const RESOURCE_MONO_PROPELLANT: &str = "MonoPropellant";
pub const RESOURCE_ABLATOR: &str = "Ablator";
pub const RESOURCE_SOLID_FUEL: &str = "SolidFuel";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct KrpcConfig {
    pub client_name: String,
    pub address: String,
    pub rpc_port: u16,
    pub telemetry_hz: f32,
}

impl Default for KrpcConfig {
    fn default() -> Self {
        Self {
            client_name: "KTYTOI SPYTNIK NAXYI".to_owned(),
            address: "127.0.0.1".to_owned(),
            rpc_port: 50_000,
            telemetry_hz: 20.0,
        }
    }
}

pub struct Krpc {
    _client: ClientRef,
}

impl Krpc {
    pub async fn connect(config: KrpcConfig) -> Result<Self> {
        let client = Client::connect(
            &config.client_name,
            &config.address,
            config.rpc_port,
        )
        .await?
        .into_shared();

        Ok(Self {
            _client: client,
        })
    }
}