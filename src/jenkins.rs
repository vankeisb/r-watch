use crate::build_status::{BuildStatus, Status};

pub async fn fetch(
    server_url: &String,
    plan: &String,
    branch: &String,
    token: &Option<String>,
    user: &Option<String>,
) -> Result<BuildStatus, String> {
    let url = format!(
        "{server_url}/job/{plan}/job/{branch}/lastCompletedBuild/api/json?tree=url,building,timestamp,estimatedDuration,result,duration&depth=0"
    );
    let headers = vec![(String::from("Accept"), String::from("application/json"))];
    match user {
        Some(user) => {
            crate::utils::request_basic::<JenkinsResponse>(
                &url,
                &headers,
                user.to_owned(),
                token.to_owned(),
            )
            .await
        }
        None => crate::utils::request::<JenkinsResponse>(&url, &headers).await,
    }
    .and_then(|response| {
        let result = response.result.as_str();
        match result {
            "SUCCESS" => Ok(BuildStatus {
                status: Status::Green,
                url: response.url,
                time_info: None,
            }),
            "FAILURE" => Ok(BuildStatus {
                status: Status::Red,
                url: response.url,
                time_info: None,
            }),
            _ => Err(format!("Unhandled result {result}")),
        }
    })
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct JenkinsResponse {
    url: String,
    duration: u32,
    result: String,
    timestamp: u64,
}
