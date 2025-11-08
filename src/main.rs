mod bamboo;
mod build_status;
mod config;

use std;

use crate::{
    build_status::BuildStatus,
    config::{env_replacer, load_config},
};

static CONFIG_FILE: &str = ".bwatch.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_file = std::env::home_dir().unwrap();
    config_file.push(CONFIG_FILE);
    let content = std::fs::read_to_string(config_file).unwrap();
    let config = load_config(&content, env_replacer).unwrap();
    let futures = config
        .builds
        .into_iter()
        .map(async |x| x.fetch().await.map(|r| (r, x)));
    let joined = futures::future::join_all(futures).await;
    for r in joined.iter() {
        match r {
            Ok((
                BuildStatus {
                    status,
                    time_info: _,
                    url,
                },
                build_config,
            )) => {
                let title = build_config.get_title();
                println!("[{status:?}] {title} {url}");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}
