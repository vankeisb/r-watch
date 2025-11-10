use crate::build_status::BuildStatus;

static BASE_URL: &str = "https://circleci.com/api/v2";

#[derive(Debug, serde::Deserialize, PartialEq)]
struct CircleCIResponse {
    items: Vec<CircleCIItem>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct CircleCIItem {
    id: String,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct WorkflowResponse {
    items: Vec<WorkflowItem>,
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct WorkflowItem {
    id: String,
    status: String,
    pipeline_number: u32,
}

pub async fn fetch(
    org: &String,
    repo: &String,
    branch: &String,
    token: &Option<String>,
) -> Result<BuildStatus, String> {
    let pipeline_url = format!("{BASE_URL}/project/github/{org}/{repo}/pipeline?branch={branch}");
    let client = reqwest::Client::new();
    let get_part = client
        .get(pipeline_url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json");
    let req = match token {
        Some(token) => get_part.header("Circle-Token", token),
        None => get_part,
    };
    // let req = token
    //     .map(|token| get_part.header("Circle-Token", token))
    //     .unwrap_or(get_part);
    match req.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            if status_code != 200 {
                Err(String::from(format!("Invalid status {}", status_code)))
            } else {
                match response.json::<CircleCIResponse>().await {
                    Ok(response) => match response.items.get(0) {
                        Some(item) => {
                            let pipeline_id = &item.id;
                            let workflow_url =
                                format!("{BASE_URL}/pipeline/{pipeline_id}/workflow");
                            let get_part = client
                                .get(workflow_url)
                                .header("Accept", "application/json")
                                .header("Content-Type", "application/json");
                            let req = match token {
                                Some(token) => get_part.header("Circle-Token", token),
                                None => get_part,
                            };
                            match req.send().await {
                                Ok(response) => match response.json::<WorkflowResponse>().await {
                                    Ok(response) => match response.items.get(0) {
                                        Some(WorkflowItem {
                                            id,
                                            status,
                                            pipeline_number,
                                        }) => {
                                            let app_url = format!(
                                                "https://app.circleci.com/pipelines/github/{org}/{repo}/{pipeline_number}/workflows/{id}"
                                            );

                                            let status = status.as_str();
                                            match status {
                                                "success" => Ok(BuildStatus {
                                                    status: crate::build_status::Status::Green,
                                                    time_info: None,
                                                    url: app_url,
                                                }),
                                                "failed" | "failing" => Ok(BuildStatus {
                                                    status: crate::build_status::Status::Red,
                                                    time_info: None,
                                                    url: app_url,
                                                }),
                                                "error" => Err(String::from("build error")),
                                                _ => Err(format!("unhandled status {status}")),
                                            }
                                        }
                                        None => Err(String::from("No items in workflow response")),
                                    },
                                    Err(error) => Err(format!("Json parsing error {:?}", error)),
                                },
                                Err(err) => Err(format!("{:?}", err)),
                            }
                        }
                        None => Err(String::from("No items in response")),
                    },
                    Err(error) => Err(format!("Json parsing error {:?}", error)),
                }
            }
        }
        Err(err) => Err(format!("{:?}", err)),
    }
}
