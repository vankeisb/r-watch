
mod bamboo;
mod build_status;
mod config;

use std;

static CONFIG_FILE: &str = ".bwatch.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config_file = std::env::home_dir().unwrap();
    config_file.push(CONFIG_FILE);
    let content = std::fs::read_to_string(config_file).unwrap();
    println!("{}", content);
    let config = serde_json::from_str::<config::Config>(&content).unwrap();
    println!("config = {:?}", config);
    let futures = config.builds
        .into_iter()
        .map(async |x| x.fetch().await);
    let joined = futures::future::join_all(futures).await;
    joined.iter().for_each(|r|
        println!("Result : {:?}", r)
    );
    Ok(())
}