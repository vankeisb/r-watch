use crate::{build_status::BuildStatus, utils::encode_uri_component};

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
    org: &str,
    repo: &str,
    branch: &str,
    token: &Option<String>,
) -> Result<BuildStatus, String> {
    let branch = encode_uri_component(branch);
    let pipeline_url = format!("{BASE_URL}/project/github/{org}/{repo}/pipeline?branch={branch}");
    let mut headers = vec![
        (String::from("Accept"), String::from("application/json")),
        (String::from("Content-Type"), String::from("application/json")),
    ];
    if let Some(t) = token {
        headers.push((String::from("Circle-Token"), t.to_string()));
    }

    crate::utils::request::<CircleCIResponse>(&pipeline_url, &headers)
        .await
        .and_then(|r| match r.items.into_iter().next() {
            Some(item) => Ok(item),
            None => Err(String::from("No CI item found"))
        })
        .and_then( |item| futures::executor::block_on(async { 
            let pipeline_id = encode_uri_component(&item.id);
            let workflow_url = format!("{BASE_URL}/pipeline/{pipeline_id}/workflow");
            crate::utils::request::<WorkflowResponse>(&workflow_url, &headers)
                .await
        }))
        .and_then(|r| {
            let x = r.items.into_iter().next();
            match x {
                Some(item) => Ok(item),
                None => Err(String::from("No workflow item found"))
            }
        })
        .and_then(|workflow_item| workflow_item.to_build_status(org, repo))        
}
