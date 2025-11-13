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

impl WorkflowResponse {
    fn first_item(&self) -> Result<&WorkflowItem,String> {
        match self.items.get(0) {
            Some(item) => {                
                Ok(item)
            },
            None => Err(String::from("No item found in response"))
        }
    }
}

#[derive(Debug, serde::Deserialize, PartialEq)]
struct WorkflowItem {
    id: String,
    status: String,
    pipeline_number: u32,
}

impl WorkflowItem {
    fn to_build_status(&self, org: &str, repo: &str) -> Result<BuildStatus,String> {
        let WorkflowItem { id, status, pipeline_number } = self;
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

    match crate::utils::request::<CircleCIResponse>(&pipeline_url, &headers).await {
        Ok(response) => match response.items.get(0) {
            Some(item) => {
                let pipeline_id = &item.id;
                let workflow_url = format!("{BASE_URL}/pipeline/{pipeline_id}/workflow");
                crate::utils::request::<WorkflowResponse>(&workflow_url, &headers)
                    .await
                    .and_then(|r| {
                        match r.first_item() {
                            Ok(item) => {
                                item.to_build_status(org, repo)
                            },
                            Err(e) => Err(e)
                        }
                    })                
            }
            None => Err(String::from("No items in response")),
        },
        Err(e) => Err(e),
    }    
}
