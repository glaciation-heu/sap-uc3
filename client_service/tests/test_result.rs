mod common;

#[cfg(test)]
mod test {
    use core::time;
    use std::thread;

    use reqwest::StatusCode;
    use tokio;

    //const RESULT_UUID: &str = "00000000-0000-0000-0000-000000000000";

    use crate::common;
    /// Testing get results
    #[tokio::test]
    async fn test_get_result_success() {
        let collab = common::setup_env().await;
        let client = common::test_client();
        common::register_input_party(collab.id, 1).await;

        // Upload secret to start execution
        let upload_resp = common::upload_secret(&client, collab.id, 1, None).await;

        upload_resp.assert_status_is_ok();

        // get result
        for _ in 0..5 {
            // Sleep two seconds to wait for execution to be finished.
            thread::sleep(time::Duration::from_secs(2));

            let resp= client.get(format!("/result/{}/1", collab.id))
                .send()
                .await;
            // proccessing not finished yet.
            if resp.0.status() == StatusCode::CONFLICT {
                continue;
            }
            resp.assert_status_is_ok();
            break;
        }
    }

    #[tokio::test]
    async fn test_get_result_no_collab() {
        let collab = common::setup_env().await;
        let client = common::test_client();
        let resp = client.get(format!("/result/{}/2", collab.id+1))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
        let body = resp.0.into_body().into_string().await;
        assert!(true, "response: {:?}", body);
    }
}