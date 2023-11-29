use std::process::exit;
use sslb::lb::LoadBalancer;
use sslb::policy::SimpleRoundRobinPolicy;
use sslb::config::LbConfig;
use log::{info, error};

const CONFIG_FILENAME: &'static str = "sslb.toml";

#[tokio::main]
async fn main() {
    env_logger::init();
    let toml = match LbConfig::build(CONFIG_FILENAME) {
        Ok(c) => c,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };

    let policy =
        Box::new(SimpleRoundRobinPolicy::new(toml.config.endpoints.into_iter().collect()));

    info!("Building load balancer...");
    let mut server = match LoadBalancer::build(
        &toml.config.ip,
        policy).await {
        Ok(s) => s,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };
    info!("Load balancer built.");
    info!("Running load balancer...");
    if let Err(err) = server.run().await {
        error!("{}", err);
    }
}
