use stayputnik::{
    Client,
    ClientRef,
    services::{
        krpc::{
            GameScene,
            KRPC,
        },
        space_center::{
            SpaceCenter,
            Vessel,
        },
    },
};

use super::{
    config::KrpcConfig,
    error::{
        KrpcError,
        Result,
    },
};

pub struct KrpcSession {
    _client: ClientRef,
    core: KRPC,
    space_center: SpaceCenter,
}

impl KrpcSession {
    pub async fn connect(
        config: &KrpcConfig,
    ) -> Result<Self> {
        let client = Client::connect(
            &config.client_name,
            &config.address,
            config.rpc_port,
        )
        .await?
        .into_shared();

        let core = KRPC::new(client.clone());

        let space_center =
            SpaceCenter::new(client.clone());

        let session = Self {
            _client: client,
            core,
            space_center,
        };

        session.ensure_flight_scene().await?;

        Ok(session)
    }

    pub async fn active_vessel(
        &self,
    ) -> Result<Vessel> {
        self.ensure_flight_scene().await?;

        Ok(
            self.space_center
                .active_vessel()
                .await?,
        )
    }

    pub async fn ensure_flight_scene(
        &self,
    ) -> Result<()> {
        let scene = self
            .core
            .current_game_scene()
            .await?;

        if scene != GameScene::Flight {
            return Err(
                KrpcError::NotInFlight(scene),
            );
        }

        Ok(())
    }

    pub fn space_center(&self) -> &SpaceCenter {
        &self.space_center
    }
}