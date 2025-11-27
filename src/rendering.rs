use std::collections::HashMap;

use pad::PadStr;
use terminal_hyperlink::Hyperlink;

use crate::{
    build_status::{BuildStatus, Status},
    config::BuildConfig,
};

enum Row {
    SuccessRow { 
        status: char,
        title: String, 
        url: String, 
        completed_at: String,
        duration: String,
    },
    FailureRow {
        title: String,
        error: String,
        url: String,
    }
}

impl Row {
    fn get_title(&self) -> &String {
        match self {
            Row::SuccessRow { status: _, title, url: _,  completed_at: _, duration: _} => title,
            Row::FailureRow { title, error: _, url: _} => title,
        }
    }
} 

const STATUS_GREEN: char = '✅';
const STATUS_RED: char = '❌';
const STATUS_ERR: char = '💣';

fn status_to_string(status: &Status) -> char {
    match status {
        Status::Green => STATUS_GREEN,
        Status::Red => STATUS_RED,
    }
}

#[derive(Debug)]
struct GroupData {
    nb_green: u32,
    nb_red: u32,
    nb_err: u32,
}

pub fn render_groups(rows: Vec<(BuildConfig, Result<BuildStatus,String>)>) -> () {
    let mut by_tags: HashMap<String, GroupData> = HashMap::new();
    let mut max_group = 0;
    for row in rows.into_iter() {
        for group in row.0.get_groups() {
            max_group = std::cmp::max(group.len(), max_group);
            let data = by_tags.entry(group).or_insert(GroupData {
                nb_green: 0,
                nb_red: 0,
                nb_err: 0,
            });
            match &row.1 {
                Ok(x) => {
                    match x.status {
                        Status::Green => {
                            data.nb_green = data.nb_green + 1;
                        },
                        Status::Red => {
                            data.nb_red = data.nb_red + 1;
                        }
                    }
                },
                Err(_) => {
                    data.nb_err = data.nb_err + 1;
                }
            }
        }        
    }

    let mut sorted_keys: Vec<&String> = by_tags.keys().into_iter().collect();//.into_iter().collect();
    sorted_keys.sort();
    for key in sorted_keys.iter() {
        let data = by_tags.get(*key);
        let key = key.pad_to_width(max_group);
        match data {
            Some(data) => {
                let nb_green = pad_num(data.nb_green);
                let nb_red = pad_num(data.nb_red);
                let nb_err = pad_num(data.nb_err);
                println!("{key} | {nb_green} {STATUS_GREEN} | {nb_red} {STATUS_RED} | {nb_err} {STATUS_ERR}");
            },
            None => {
                println!("{key} | {STATUS_ERR} no data");
            }
        }
    }
}

pub fn pad_num(n: u32) -> String {
    let s = format!("{n}");
    s.pad_to_width_with_alignment(3, pad::Alignment::Right)
}

pub fn render_rows(rows: Vec<(BuildConfig, Result<BuildStatus,String>)>) -> () {
    let mut res: Vec<Row> = Vec::new();
    let mut max_title = 0;
    let mut max_url = 0;
    let mut max_completed_at = 0;
    let mut max_duration = 0;

    for (config, fetch_result) in rows.into_iter() {
        max_title = std::cmp::max(max_title, config.get_title().len());
        match fetch_result {
            Ok(status) => {
                max_url = std::cmp::max(max_url, status.url.len());
                let (completed_at, duration) = status
                    .time_info
                    .as_ref()
                    .map(|time_info| {
                        let parsed_date = chrono::DateTime::parse_from_rfc3339(&time_info.completed_at)
                            .map(|parsed_date| parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
                            .ok()
                            .unwrap_or(time_info.completed_at.to_string());
                        let secs: u64 = u64::try_from(time_info.duration_secs).unwrap();
                        let d = std::time::Duration::from_secs(secs);
                        let pretty = pretty_duration::pretty_duration(&d, None);
                        (parsed_date, pretty)
                    })
                    .unwrap_or((String::new(), String::new()));

                max_completed_at = std::cmp::max(max_completed_at, completed_at.len());
                max_duration = std::cmp::max(max_duration, duration.len());

                res.push(Row::SuccessRow {
                    status: status_to_string(&status.status),
                    title: config.get_title(),
                    url: status.url.to_string(),
                    completed_at,
                    duration,
                });
            },
            Err(e) => {
                res.push(Row::FailureRow { 
                    title: config.get_title(),
                    error: e,
                    url: config.get_url(),
                });
            },
        }
    }

    res.sort_by(|a, b| a.get_title().cmp(&b.get_title()));
    for row in res.into_iter() {        
        match row {
            Row::SuccessRow { status, title, url, completed_at, duration } => {
                let title = &title.pad_to_width(max_title);
                let clickable_title = title.hyperlink(url);
                let completed_at = completed_at
                    .pad_to_width_with_alignment(max_completed_at, pad::Alignment::Right);
                let duration = duration
                    .pad_to_width_with_alignment(max_duration, pad::Alignment::Right);
                let row = format!("{status} {clickable_title} | {completed_at} | {duration}");
                println!("{row}");
            },
            Row::FailureRow { title, error, url } => {
                let title = &title.pad_to_width(max_title);
                let clickable_title = title.hyperlink(url);
                let row = format!("{STATUS_ERR} {clickable_title} | {error}");
                println!("{row}");
            },
        }        
    }
}