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
    let mut headers = vec![
        (String::from("Accept"), String::from("application/json")),
        (String::from("Content-Type"), String::from("application/json")),
    ];
    if let Some(t) = token {
        headers.push((String::from("Circle-Token"), t.to_string()));
    }

    let response = crate::utils::request::<CircleCIResponse>(&pipeline_url, &headers).await;
    match response {
        Ok(response) => match response.items.get(0) {
            Some(item) => {
                let pipeline_id = &item.id;
                let workflow_url = format!("{BASE_URL}/pipeline/{pipeline_id}/workflow");

                match crate::utils::request::<WorkflowResponse>(&workflow_url, &headers).await {
                    Ok(response) => 
                        match response.items.get(0) {
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
                        }
                    Err(e) => Err(e),
                }
            }
            None => Err(String::from("No items in response")),
        },
        Err(e) => Err(e),
    }    
}
