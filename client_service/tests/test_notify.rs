mod common;

#[cfg(test)]
mod test {
    use reqwest::StatusCode;
    use tokio;

    use crate::common;
    /// Testing the /notify endpoint
    #[tokio::test]
    async fn test_put_notify() {
        let client = common::test_client();
        let request_body = serde_json::json!({
            "message": "testing notify",
            "code": 200,
            "collaborationId": 123,
            "secretId": "abc-123"
        });
        let resp = client.put("/notify")
            .body_json(&request_body)
            .send()
            .await;
        resp.assert_status(StatusCode::ACCEPTED);
    }
}