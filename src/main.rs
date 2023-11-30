use std::process::exit;
use sslb::{lb::LoadBalancer, policy::PolicyType};
use sslb::policy::create_policy;
use sslb::config::LbConfig;
use log::{info, error};

const CONFIG_FILENAME: &'static str = "sslb.toml";

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let toml = match LbConfig::build(CONFIG_FILENAME) {
        Ok(c) => c,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };

    let policy_type = match PolicyType::try_from(toml.config.policy.clone()) {
        Ok(p) => p,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };
    info!("Using policy \"{}\".", toml.config.policy);
    let policy = create_policy(policy_type, toml.config.endpoints);

    info!("Building load balancer...");
    let mut server = match LoadBalancer::build(
        &toml.config.addr,
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
