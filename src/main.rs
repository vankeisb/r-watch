mod bamboo;
mod build_status;
mod circle_ci;
mod config;
mod jenkins;
mod rendering;
mod travis;
mod utils;

use crate::{
    build_status::BuildStatus,
    config::{BuildConfig, env_replacer, load_config},
    rendering::render_rows,
};
use clap::Parser;
use std;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    filter: Option<String>,
}

static CONFIG_FILE: &str = ".bwatch.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config_file = std::env::home_dir().unwrap();
    config_file.push(CONFIG_FILE);
    let content = std::fs::read_to_string(config_file).unwrap();
    let config = load_config(&content, env_replacer).unwrap();
    let futures = config
        .builds
        .into_iter()
        .filter(|build| match &cli.filter {
            Some(filter) => build.get_title().contains(filter),
            None => true,
        })
        .map(async |x| match x.fetch().await {
            Ok(r) => Ok((r, x)),
            Err(e) => Err((e, x)),
        });
    let joined = futures::future::join_all(futures).await;
    let mut rows: Vec<(&BuildConfig, &BuildStatus)> = Vec::new();
    for r in joined.iter() {
        match r {
            Ok((status, config)) => {
                rows.push((config, status));
            }
            Err((e, config)) => {
                println!("💣 {} {:?}", config.get_title(), e);
            }
        }
    }
    render_rows(rows);
    Ok(())
}
