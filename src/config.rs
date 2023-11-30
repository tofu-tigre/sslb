use std::{error::Error, fs};
use serde_derive::Deserialize;
use toml;

#[derive(Deserialize)]
pub struct LbConfig {
  pub config: Config,
}

#[derive(Deserialize)]
pub struct Config {
  pub addr: String,
  pub policy: String,
  pub endpoints: Vec<String>,
}

impl LbConfig {
  pub fn build(src: &str) -> Result<Self, Box<dyn Error>> {
    // Open the config file.
    let contents = fs::read_to_string(src)?;
    let data: LbConfig = toml::from_str(&contents)?;
    Ok(data)
  }
}