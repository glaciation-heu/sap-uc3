use poem_openapi::{payload::Json, types::ParseFromJSON, ApiResponse};
use crate::{cs_client::{ClearTextSecret, CsClient}, error::{Error, Result}, netaccess::NetAccess};
use tracing::{event, Level};

use super::utils::coordinator_uri;

#[derive(ApiResponse, Debug)]
pub enum ResultResponse {
    /// Computation ID response
    #[oai(status = 200)]
    ComputationResult(Json<Vec<ClearTextSecret>>),
}

pub async fn result(collab_id: i32, _party_id: i32, cs_client: &impl CsClient, net: &impl NetAccess) -> Result<ResultResponse> {
    let result_ids = get_result_ids(collab_id, net).await?;
    let mut secrets: Vec<ClearTextSecret> = vec![];
    for id in result_ids {
        let res = cs_client.get_secret(&id).await?;
        secrets.push(res);
    }
    Ok(ResultResponse::ComputationResult(Json(secrets)))
}

async fn get_result_ids(collab_id: i32, net: &impl NetAccess) -> Result<Vec<String>> {
    let url = format!("{}/collaboration/{}/result_ids", coordinator_uri(), collab_id);
    let res = net.get(&url).await?;
    let s = String::from_utf8_lossy(&res);
    event!(Level::DEBUG, "Try parsing json {}", &s);
    match ParseFromJSON::parse_from_json_string(&s) {
        Ok(res) => Ok(res),
        Err(err) => {
            event!(Level::WARN, "Unable to parse json: {}", &s);
            Err(Error::from(format!("{:?}", err)))
        }
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use tokio_test::assert_err;

    use crate::{cs_client::MockCsClient, netaccess::MockNetAccess};

    use super::*;

    #[tokio::test]
    async fn test_get_result_ids() {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Ok("[\"asdfa\"]".as_bytes().to_vec())
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let res = get_result_ids(1, &net).await;
        net.checkpoint();
        match res {
            Ok(res) => assert_eq!(res, vec!["asdfa"]),
            Err(err) => panic!("Did not expect error for this test: {}", err)
        }
    }

    #[tokio::test]
    async fn test_get_result_ids_failing() {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Err(Error::from("Unable to get collaboration with id"))
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let res = get_result_ids(1, &net).await;
        net.checkpoint();
        match res {
            Ok(_) => assert!(false, "This test should return en error"),
            Err(_) => assert!(true),
        }
    }

    #[tokio::test]
    async fn test_get_result() -> Result<()> {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Ok("[\"asdf\"]".as_bytes().to_vec())
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let mut client = MockCsClient::new();
        client.expect_get_secret()
            .times(1)
            .returning(|_| {
                Ok(ClearTextSecret {
                    result: "asdf".to_string(),
                    // creation_date: None,
                    // game_id: None,
                })
            });
        let ResultResponse::ComputationResult(res) = result(1, 1, &client, &net).await?;
        net.checkpoint();
        client.checkpoint();
        assert_eq!(res.0.len(), 1);
        assert_eq!(res.0[0].result, "asdf");
        Ok(())
    }


    #[tokio::test]
    async fn test_get_result_ids_failing_json() {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Ok("\"asdfa\"]".as_bytes().to_vec())
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let res = get_result_ids(1, &net).await;
        net.checkpoint();
        match res {
            Ok(_) => assert!(false, "This test should not return OK, because of invalid json"),
            Err(_) => assert!(true)
        }

    }

    #[tokio::test]
    async fn test_get_result_failing_net() -> Result<()> {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Err(Error::from(""))
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        env::set_var("COORDINATOR_URI", "http://coordinator");
        let mut client = MockCsClient::new();
        client.expect_get_secret()
            .times(0);
        let res = result(1, 1, &client, &net).await;
        assert_err!(res);
        net.checkpoint();
        client.checkpoint();
        Ok(())
    }
}