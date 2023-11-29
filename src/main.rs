use std::process::exit;
use sslb::lb::LoadBalancer;
use sslb::policy::SimpleRoundRobinPolicy;
use sslb::config::LbConfig;

const CONFIG_FILENAME: &'static str = "sslb.toml";

#[tokio::main]
async fn main() {
    let toml = match LbConfig::build(CONFIG_FILENAME) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    };

    let policy =
        Box::new(SimpleRoundRobinPolicy::new(toml.config.endpoints));

    let mut server = match LoadBalancer::build(
        &toml.config.ip,
        policy).await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Error: {}", err);
            exit(1);
        }
    };
    server.run().await;
}
