use terminal_hyperlink::Hyperlink;

use crate::{
    build_status::{BuildStatus, Status},
    config::BuildConfig,
};

pub struct RowData {
    status: char,
    title: String,
    url: String,
    completed_at: String,
    duration: String,
}

const STATUS_GREEN: char = '✅';
const STATUS_RED: char = '❌';

fn status_to_string(status: &Status) -> char {
    match status {
        Status::Green => STATUS_GREEN,
        Status::Red => STATUS_RED,
    }
}

fn pad_str(s: &str, i: usize) -> String {
    format!("{:width$}", s, width = i)
}

pub fn render_rows(rows: Vec<(&BuildConfig, &BuildStatus)>) -> () {
    let mut res: Vec<RowData> = Vec::new();
    let mut max_title = 0;
    let mut max_url = 0;
    let mut max_completed_at = 0;
    let mut max_duration = 0;

    for (config, status) in rows.into_iter() {
        max_title = std::cmp::max(max_title, config.get_title().len());
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

        res.push(RowData {
            status: status_to_string(&status.status),
            title: config.get_title(),
            url: status.url.to_string(),
            completed_at,
            duration,
        });
    }

    res.sort_by(|a, b| a.title.cmp(&b.title));
    for row in res.into_iter() {
        let status = row.status;
        let title = pad_str(&row.title, max_title);
        let clickable_title = title.hyperlink(&row.url);
        let completed_at = pad_str(&row.completed_at, max_completed_at);
        let duration = pad_str(&row.duration, max_duration);
        let row = format!("{status} {clickable_title} | {completed_at} | {duration}");
        println!("{row}");
    }

    ()
}
