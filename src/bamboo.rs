use crate::build_status::{BuildStatus, TimeInfo, Status};


#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BambooResponse {
    results: BambooResults
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BambooResults {
    size: usize,
    result: Vec<BambooResult>
}

#[derive(Debug, serde::Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BambooResult {
    build_state: String,
    life_cycle_state: String,
    build_result_key: String,
    build_completed_time: String,
    build_duration: u32,
}

impl BambooResponse {
    fn to_build_status(&self, server_url: &String) -> Option<BuildStatus> {
        self.results.result
            .get(0)
            .filter(|result| result.life_cycle_state == "Finished")
            .map(|result| {
                let status = if result.build_state == "Successful" { Status::Green } else { Status::Red };
                let server_url = server_url;
                let build_result_key = &result.build_result_key;
                let url = format!("{server_url}/browse/{build_result_key}");
                let time_info = TimeInfo {
                    completed_at: result.build_completed_time.clone(),
                    duration_secs: result.build_duration / 1000,
                };
                BuildStatus {
                    status,
                    url,
                    time_info: Some(time_info),
                }
            })
    }
}

pub async fn fetch(server_url: &String, plan: &String, token: &Option<String>) -> Result<BuildStatus, String> {
    let url = format!("{server_url}/rest/api/latest/result/{plan}.json?max-results=1&expand=results.result");
    let client = reqwest::Client::new();
    let get_part = client.get(url);
    let req_builder = match token {
        Some(t) => get_part.header("Authorization", format!("Bearer {t}")),
        None => get_part
    };
    let resp: Result<reqwest::Response, reqwest::Error> = req_builder
        .send()
        .await;

    match resp {
        Ok(response) => {
            let status = response.status().as_u16();
            if status == 200 {
                // let j = response.text().await;
                match response.json::<BambooResponse>().await {
                    Ok(response) => {
                        match response.to_build_status(server_url) {
                            Some(build_status) => {
                                Ok(build_status)
                            }
                            None => {
                                Err(String::from("No build found in response"))
                            }
                        }
                    }
                    Err(error) => {
                        Err(format!("Json parsing error {:?}", error))
                    }
                }                        
            } else {
                Err(format!("Invalid status code {status}"))
            }
        },
        Err(err) => {
            let msg = format!("{:?}", err);
            Err(msg)
        }
    }
}

#[cfg(test)]
mod bamboo_tests {
    use super::*;

    #[test]
    fn decode_response() {
        let s = String::from("{\"results\":{\"size\":1,\"expand\":\"result\",\"start-index\":0,\"max-result\":1,\"result\":[{\"expand\":\"plan,vcsRevisions,artifacts,comments,labels,jiraIssues,variables,stages\",\"link\":{\"href\":\"https://sfactory.francelab.fr.ibm.com:8443/rest/api/latest/result/TRUNK-DTRTMP-2203\",\"rel\":\"self\"},\"plan\":{\"shortName\":\"Studio Tests DTR\",\"shortKey\":\"DTRTMP\",\"type\":\"chain\",\"enabled\":true,\"link\":{\"href\":\"https://sfactory.francelab.fr.ibm.com:8443/rest/api/latest/plan/TRUNK-DTRTMP\",\"rel\":\"self\"},\"key\":\"TRUNK-DTRTMP\",\"name\":\"JRules trunk - Studio Tests DTR\",\"planKey\":{\"key\":\"TRUNK-DTRTMP\"}},\"planName\":\"Studio Tests DTR\",\"projectName\":\"JRules trunk\",\"buildResultKey\":\"TRUNK-DTRTMP-2203\",\"lifeCycleState\":\"Finished\",\"id\":412588364,\"buildStartedTime\":\"2025-11-07T08:58:40.000+01:00\",\"prettyBuildStartedTime\":\"Fri, 7 Nov, 08:58 AM\",\"buildCompletedTime\":\"2025-11-07T09:19:46.000+01:00\",\"buildCompletedDate\":\"2025-11-07T09:19:46.000+01:00\",\"prettyBuildCompletedTime\":\"Fri, 7 Nov, 09:19 AM\",\"buildDurationInSeconds\":1266,\"buildDuration\":1266000,\"buildDurationDescription\":\"21 minutes\",\"buildRelativeTime\":\"1 hour ago\",\"vcsRevisionKey\":\"1be5db0668982c43300eaf9c88596c10a9992ed2\",\"vcsRevisions\":{\"size\":1,\"start-index\":0,\"max-result\":1},\"buildTestSummary\":\"1 of 197 failed\",\"successfulTestCount\":196,\"failedTestCount\":1,\"quarantinedTestCount\":0,\"skippedTestCount\":0,\"continuable\":false,\"onceOff\":false,\"restartable\":true,\"notRunYet\":false,\"finished\":true,\"successful\":false,\"buildReason\":\"Changes by <a href=\\\"https://sfactory.francelab.fr.ibm.com:8443/users/viewUserSummary.action?currentUserName=fwagner\\\">Frank Wagner</a>\",\"reasonSummary\":\"Changes by <a href=\\\"https://sfactory.francelab.fr.ibm.com:8443/users/viewUserSummary.action?currentUserName=fwagner\\\">Frank Wagner</a>\",\"specsResult\":false,\"artifacts\":{\"size\":0,\"start-index\":0,\"max-result\":0},\"comments\":{\"size\":0,\"start-index\":0,\"max-result\":0},\"labels\":{\"size\":0,\"start-index\":0,\"max-result\":0},\"jiraIssues\":{\"size\":0,\"start-index\":0,\"max-result\":0},\"variables\":{\"size\":62,\"start-index\":0,\"max-result\":62},\"stages\":{\"size\":1,\"start-index\":0,\"max-result\":1},\"key\":\"TRUNK-DTRTMP-2203\",\"planResultKey\":{\"key\":\"TRUNK-DTRTMP-2203\",\"entityKey\":{\"key\":\"TRUNK-DTRTMP\"},\"resultNumber\":2203},\"state\":\"Failed\",\"buildState\":\"Failed\",\"number\":2203,\"buildNumber\":2203}]},\"expand\":\"results\",\"link\":{\"href\":\"https://sfactory.francelab.fr.ibm.com:8443/rest/api/latest/result/TRUNK-DTRTMP\",\"rel\":\"self\"}}");
        let v = serde_json::from_str::<BambooResponse>(&s).unwrap();        
        let expected = BambooResponse {
            results: BambooResults { 
                size: 1,
                result: vec!(
                    BambooResult {
                        build_state: String::from("Failed"),
                        life_cycle_state: String::from("Finished"),
                        build_result_key: String::from("TRUNK-DTRTMP-2203"),
                        build_completed_time: String::from("2025-11-07T09:19:46.000+01:00"),
                        build_duration: 1266000,
                    }
                )
            }
        };
        assert_eq!(v, expected);
    }

    #[test]
    fn convert_result() {
        let response = BambooResponse {
            results: BambooResults { 
                size: 1,
                result: vec!(
                    BambooResult {
                        build_state: String::from("Failed"),
                        life_cycle_state: String::from("Finished"),
                        build_result_key: String::from("TRUNK-DTRTMP-2203"),
                        build_completed_time: String::from("2025-11-07T09:19:46.000+01:00"),
                        build_duration: 1266000,
                    }
                )
            }
        };
        let expected = BuildStatus {
            status: Status::Red,
            url: String::from("http://my.bamboo/browse/TRUNK-DTRTMP-2203"),
            time_info: Some(
                TimeInfo {
                    completed_at: String::from("2025-11-07T09:19:46.000+01:00"),
                    duration_secs: 1266,
                }
            )
        };
        let url = String::from("http://my.bamboo");
        assert_eq!(response.to_build_status(&url).unwrap(), expected);
    }
}