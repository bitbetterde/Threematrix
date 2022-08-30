use std::env::var;
use std::fs::read_to_string;

use matrix_sdk::Client;
use serde_derive::{Deserialize, Serialize};
use tokio::sync::Mutex;

use threema::types::Message;
use crate::threema::ThreemaClient;

pub mod errors;
pub mod matrix;
pub mod threema;
pub mod util;
pub mod incoming_message_handler;

pub struct AppState {
    pub threema_client: ThreemaClient,
    pub matrix_client: Mutex<Client>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreemaConfig {
    pub secret: String,
    pub private_key: String,
    pub gateway_own_id: String,
    pub port: Option<u16>,
    pub host: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatrixConfig {
    pub homeserver_url: String,
    pub user: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggerConfig {
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreematrixConfig {
    pub threema: ThreemaConfig,
    pub matrix: MatrixConfig,
    pub logger: Option<LoggerConfig>,
}

impl ThreematrixConfig {
    pub fn new(path: &str) -> ThreematrixConfig {
        let toml_string = read_to_string(path).expect("Could not read config file");
        let mut config_from_file: ThreematrixConfig =
            toml::from_str(&toml_string).expect("Could not parse config file");

        // Get host and port from environment (e.g. for Docker use)
        let host_from_env = var("THREEMATRIX_LISTEN_HOST");
        let port_from_env = var("THREEMATRIX_LISTEN_PORT");
        if let Ok(host_from_env) = host_from_env {
            config_from_file.threema.host = Some(host_from_env)
        };
        if let Ok(port_from_env) = port_from_env {
            config_from_file.threema.port =
                Some(port_from_env.parse::<u16>().expect("Invalid Port in environment"))
        };
        return config_from_file;
    }
}