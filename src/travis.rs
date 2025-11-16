use crate::build_status::{BuildStatus, Status, TimeInfo};

fn api_url(server_url: &str) -> String {
    // console.log("serverUrl", serverUrl);
    if server_url == "https://travis-ci.org" {
        return String::from("https://api.travis-ci.org");
    }
    return format!("{server_url}/api");
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct TravisResponse {
    last_build: Option<TravisBuild>,
    error_message: Option<String>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct TravisBuild {
    state: String,
    id: u32,
    previous_state: String,
    finished_at: String,
    duration: u32,
}

fn encode_uri_component(s: &str) -> String {
    s.replace("/", "%2F")
}

pub async fn fetch(
    server_url: &String,
    repository: &String,
    branch: &String,
    token: &Option<String>,
) -> Result<BuildStatus, String> {
    let api_url = api_url(server_url);
    let repository = encode_uri_component(repository);
    let branch = encode_uri_component(branch);
    let url = format!("{api_url}/repo/{repository}/branch/{branch}");
    let mut headers = vec![
        (String::from("Accept"), String::from("application/json")),
        (
            String::from("Content-Type"),
            String::from("application/json"),
        ),
        (String::from("Travis-API-Version"), String::from("3")),
    ];
    if let Some(t) = token {
        headers.push((String::from("Authorization"), format!("token {t}")));
    }
    crate::utils::request::<TravisResponse>(&url, &headers)
        .await
        .and_then(|response| match response.last_build {
            Some(TravisBuild {
                id,
                state,
                previous_state,
                finished_at,
                duration,
            }) => {
                let url = format!("{server_url}/{repository}/builds/{id}");
                let mut state = state.as_str();
                if state == "started" || state == "created" {
                    state = &previous_state;
                }
                let time_info = Some(TimeInfo {
                    completed_at: finished_at,
                    duration_secs: duration,
                });
                match state {
                    "passed" => Ok(BuildStatus {
                        status: Status::Green,
                        url,
                        time_info,
                    }),
                    "failed" | "errored" => Ok(BuildStatus {
                        status: Status::Red,
                        url,
                        time_info,
                    }),
                    _ => Err(format!("unhandled state : {state}")),
                }
            }
            None => match response.error_message {
                Some(error_message) => Err(error_message),
                None => Err(String::from("No error message available")),
            },
        })
}
