mod bamboo;
mod build_status;
mod circle_ci;
mod config;
mod jenkins;
mod rendering;
mod travis;
mod utils;

use crate::{
    config::{env_replacer, load_config},
    rendering::{render_groups, render_rows},
};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    filter: Option<String>,

    #[arg(long)]
    groups: bool,    
}

static CONFIG_FILE: &str = ".bwatch.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config_file = std::env::home_dir().unwrap();
    config_file.push(CONFIG_FILE);
    let content = std::fs::read_to_string(config_file).unwrap();
    let config = load_config(&content, env_replacer).unwrap();
    let configs = config
        .builds
        .into_iter()
        .filter(|build| match &cli.filter {
            Some(filter) => {
                if build.get_title().contains(filter) {
                    true
                } else {
                    for group in build.get_groups().iter() {
                        if group.contains(filter) {
                            return true;
                        }
                    }
                    false
                }
            },
            None => true,
        });

    let futures = configs
        .map(async |x| 
            match x.fetch().await {
                Ok(r) => (x, Ok(r)),
                Err(e) => (x, Err(e))
            }
        );
    let joined = futures::future::join_all(futures).await;

    if cli.groups {
        render_groups(joined);
    } else {
        render_rows(joined);
    }
    Ok(())
}