mod bamboo;
mod build_status;
mod circle_ci;
mod config;
mod utils;
mod rendering;

use std;

use crate::{
    build_status::{BuildStatus},
    config::{BuildConfig, env_replacer, load_config}, rendering::render_rows,
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

    let mut rows: Vec<(&BuildConfig, &BuildStatus)> = Vec::new();    
    for r in joined.iter() {
        match r {
            Ok((status, config)) => {
                rows.push((config, status));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    render_rows(rows);

    Ok(())
}
