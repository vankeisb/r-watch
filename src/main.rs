mod bamboo;
mod build_status;
mod config;

use std;

use crate::{
    build_status::{BuildStatus, TimeInfo},
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
                    time_info,
                    url,
                },
                build_config,
            )) => {
                let title = build_config.get_title();
                let elapsed: String = match time_info {
                    Some(TimeInfo {
                        completed_at,
                        duration_secs,
                    }) => {
                        let parsed_date = chrono::DateTime::parse_from_rfc3339(completed_at)
                            .map(|parsed_date| parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
                            .ok()
                            .unwrap_or(completed_at.to_string());
                        let secs: u64 = u64::try_from(*duration_secs).unwrap();
                        let d = std::time::Duration::from_secs(secs);
                        let pretty = pretty_duration::pretty_duration(&d, None);
                        format!("{} | {}", parsed_date, pretty)
                    }
                    None => String::from(""),
                };
                let status = match status {
                    build_status::Status::Green => '🟩',
                    build_status::Status::Red => '🟥',
                };
                println!("{status} {title} | {url} | {elapsed}");
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    Ok(())
}
