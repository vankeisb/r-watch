use futures::future::Either;
use pad::PadStr;
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
    }
}

impl Row {
    fn get_title(&self) -> &String {
        match self {
            Row::SuccessRow { status: _, title, url: _,  completed_at: _, duration: _} => title,
            Row::FailureRow { title, error: _} => title,
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

pub fn render_tags(rows: Vec<(&BuildConfig, &BuildStatus)>) -> () {
    let mut by_tags: std::collections::HashMap<&String, &(&BuildConfig, &BuildStatus)> =
        std::collections::HashMap::new();
    for x in rows.iter() {
        let groups = x.0.get_groups().iter().flatten();
        for group in groups {
            by_tags.insert(group, x);
        }
    }

    ()
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
                })
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
            Row::FailureRow { title, error } => {
                let title = &title.pad_to_width(max_title);
                let row = format!("{STATUS_ERR} {title} | {error}");
                println!("{row}");
            },
        }        
    }

    ()

}

// pub fn render_rows(rows: Vec<(&BuildConfig, &BuildStatus)>) -> () {
//     let mut res: Vec<RowData> = Vec::new();
//     let mut max_title = 0;
//     let mut max_url = 0;
//     let mut max_completed_at = 0;
//     let mut max_duration = 0;

//     for (config, status) in rows.into_iter() {
//         max_title = std::cmp::max(max_title, config.get_title().len());
//         max_url = std::cmp::max(max_url, status.url.len());

//         let (completed_at, duration) = status
//             .time_info
//             .as_ref()
//             .map(|time_info| {
//                 let parsed_date = chrono::DateTime::parse_from_rfc3339(&time_info.completed_at)
//                     .map(|parsed_date| parsed_date.format("%Y-%m-%d %H:%M:%S").to_string())
//                     .ok()
//                     .unwrap_or(time_info.completed_at.to_string());
//                 let secs: u64 = u64::try_from(time_info.duration_secs).unwrap();
//                 let d = std::time::Duration::from_secs(secs);
//                 let pretty = pretty_duration::pretty_duration(&d, None);
//                 (parsed_date, pretty)
//             })
//             .unwrap_or((String::new(), String::new()));

//         max_completed_at = std::cmp::max(max_completed_at, completed_at.len());
//         max_duration = std::cmp::max(max_duration, duration.len());

//         res.push(RowData {
//             status: status_to_string(&status.status),
//             title: config.get_title(),
//             url: status.url.to_string(),
//             completed_at,
//             duration,
//         });
//     }

//     res.sort_by(|a, b| a.title.cmp(&b.title));
//     for row in res.into_iter() {
//         let status = row.status;
//         let title = &row.title.pad_to_width(max_title);
//         let clickable_title = title.hyperlink(&row.url);
//         let completed_at = &row
//             .completed_at
//             .pad_to_width_with_alignment(max_completed_at, pad::Alignment::Right);
//         let duration = &row
//             .duration
//             .pad_to_width_with_alignment(max_duration, pad::Alignment::Right);
//         let row = format!("{status} {clickable_title} | {completed_at} | {duration}");
//         println!("{row}");
//     }

//     ()
// }
