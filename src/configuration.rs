use secrecy::Secret;
use serde_aux::field_attributes::{deserialize_bool_from_anything, deserialize_number_from_string};

#[derive(serde::Deserialize, Clone)]
pub struct Server {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct Collectors {
    // /ip/firewall/
    //// ip_firewall enables collection, each sub command needs to also be enabled
    pub ip_firewall: Option<bool>,
    pub ip_firewall_filter: Option<bool>,
    pub ip_firewall_nat: Option<bool>,
    pub ip_firewall_mangle: Option<bool>,
    pub ip_firewall_raw: Option<bool>,
    pub ip_firewall_conntrack: Option<bool>,

    // /system/health/
    pub health: Option<bool>,

    // /system/resources/
    pub resources: Option<bool>,

    // /interfaces
    pub interfaces: Option<bool>,
    // /interfaces/poe
    pub interfaces_poe: Option<bool>,
    // /interfaces/ethernet/monitor
    pub interfaces_monitor: Option<bool>,
}

#[derive(serde::Deserialize, Clone)]
pub struct RouterConfiguration {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub check_ssl: bool,
    pub address: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub collectors: Option<Collectors>,
}

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub server: Server,
    pub metrics_prefix: String,

    pub defaults: RouterConfiguration,

    pub instances: Option<Vec<RouterConfiguration>>,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();

    // TODO: Allow to specify path, for now /config
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("config");
    let root_configuration_directory = base_path.join("config");
    // Read the "default" configuration file from default.yaml, and fail if it can't be read
    settings.merge(config::File::with_name("default.toml").required(true))?;

    // read in /config/config.toml, don't fail if the file doesnt exist
    settings
        .merge(config::File::from(root_configuration_directory.join("config.toml")).required(false))?;

    // read in config/config.toml, don't fail if the file doesnt exist
    settings
        .merge(config::File::from(configuration_directory.join("config.toml")).required(false))?;

    // Read in any settings from ROUTEROS_*
    settings.merge(config::Environment::with_prefix("routeros").separator("__"))?;

    // generate the settings
    settings.try_into()
}
