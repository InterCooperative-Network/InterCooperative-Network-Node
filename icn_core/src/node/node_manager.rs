use crate::config::{ConfigLoader, config_loader}; // Ensure correct imports for ConfigLoader and config_loader
use crate::coordinator::ModuleCoordinator;
use icn_shared::IcnResult;
use config::Config;

pub struct NodeManager {
    config_loader: ConfigLoader,
    coordinator: ModuleCoordinator,
}

impl NodeManager {
    pub fn new(config_loader: ConfigLoader, coordinator: ModuleCoordinator) -> IcnResult<Self> {
        Ok(NodeManager {
            config_loader,
            coordinator,
        })
    }

    pub async fn start(&mut self) -> IcnResult<()> {
        let config = self.config_loader.get_config();  // Get the custom config
        let config_converted = self.convert_to_external_config(&config); // Convert to `config::Config` using reference
        self.coordinator.start(&config_converted).await?;
        Ok(())
    }

    fn convert_to_external_config(&self, config: &config_loader::Config) -> Config {
        let mut external_config = Config::builder();

        // Assuming that `host` and `port` are relevant fields you want to use for configuring
        external_config = external_config.set_override("server.host", config.server.host.clone()).unwrap();
        external_config = external_config.set_override("server.port", config.server.port.to_string()).unwrap();
        external_config = external_config.set_override("database.urls", config.database.urls.join(",")).unwrap();

        external_config.build().unwrap()
    }

    // Other methods...
}
