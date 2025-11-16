async fn send_request(
    url: &str,
    headers: &Vec<(String, String)>,
) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let mut builder = client.get(url);
    for (key, value) in headers {
        builder = builder.header(key, value);
    }
    return builder.send().await;
}

fn handle_status(
    r: Result<reqwest::Response, reqwest::Error>,
) -> Result<reqwest::Response, String> {
    r.map_err(|e| format!("Request error {:?}", e))
        .and_then(|r| {
            let status = r.status().as_u16();
            if status == 200 {
                Ok(r)
            } else {
                Err(format!("Invalid status {status}"))
            }
        })
}

async fn resp_to_json<T: serde::de::DeserializeOwned>(
    r: Result<reqwest::Response, String>,
) -> Result<T, String> {
    match r {
        Ok(response) => response
            .json::<T>()
            .await
            .map_err(|e| format!("JSON Decode error : {:?}", e)),
        Err(e) => Err(e),
    }
}

pub async fn request<T: serde::de::DeserializeOwned>(
    url: &str,
    headers: &Vec<(String, String)>,
) -> Result<T, String> {
    resp_to_json::<T>(handle_status(send_request(url, headers).await)).await
}
