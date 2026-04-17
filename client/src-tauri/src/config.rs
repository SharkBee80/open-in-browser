use serde::{Deserialize, Serialize};

pub const STORE_FILE: &str = "config.json";
pub const DEFAULT_PORT: u16 = 52798;
pub const DEFAULT_KEY: &str = "open-in-browser";

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub port: u16,
    pub key: String,
}
