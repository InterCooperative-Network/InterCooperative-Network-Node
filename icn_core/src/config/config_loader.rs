use config::{Config, ConfigError, File};

pub struct ConfigLoader {
    config: Config,
}

impl ConfigLoader {
    pub fn new() -> Self {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name("config/local").required(false))
            .build()
            .unwrap_or_else(|err| {
                eprintln!("Error loading configuration: {:?}", err);
                std::process::exit(1);
            });

        ConfigLoader { config }
    }

    pub fn get_str(&self, key: &str) -> Result<String, ConfigError> {
        self.config.get_string(key)
    }

    pub fn get_int(&self, key: &str) -> Result<i64, ConfigError> {
        self.config.get_int(key)
    }
}
