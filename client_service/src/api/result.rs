use cs_interface::{ClearTextSecret, CsClient, NetAccess};
use poem_openapi::{payload::Json, types::ParseFromJSON, ApiResponse};
use crate::{error::{Error, Result}};
use tracing::{event, Level};

#[derive(ApiResponse, Debug)]
pub enum ResultResponse {
    /// Computation ID response
    #[oai(status = 200)]
    ComputationResult(Json<Vec<ClearTextSecret>>),
}

pub async fn result(coord_uri: &str, collab_id: i32, _party_id: i32, cs_client: &impl CsClient, net: &impl NetAccess) -> Result<ResultResponse> {
    let result_ids = get_result_ids(coord_uri, collab_id, net).await?;
    let mut secrets: Vec<ClearTextSecret> = vec![];
    for id in result_ids {
        let res = cs_client.get_secret(&id)?;
        secrets.push(res);
    }
    Ok(ResultResponse::ComputationResult(Json(secrets)))
}

async fn get_result_ids(coord_uri: &str, collab_id: i32, net: &impl NetAccess) -> Result<Vec<String>> {
    let url = format!("{}/collaboration/{}/result_ids", coord_uri, collab_id);
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
    use tokio_test::assert_err;

    use cs_interface::{MockCsClient, MockNetAccess};

    use super::*;

    #[tokio::test]
    async fn test_get_result_ids() {
        let mut net = MockNetAccess::new();
        net.expect_get()
            .times(1)
            .returning(|_| {
                Ok("[\"asdfa\"]".as_bytes().to_vec())
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        let res = get_result_ids("http://coordinator", 1, &net).await;
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
                Err(cs_interface::Error::HttpError{ code: 404, message: "Unable to get collaboration with id".to_string()})
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        let res = get_result_ids( "http://coordinator", 1, &net).await;
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
        let ResultResponse::ComputationResult(res) = result("http://coordinator",1, 1, &client, &net).await?;
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
        let res = get_result_ids("http://coordinator", 1, &net).await;
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
                Err(cs_interface::Error::HttpError{ code: 404, message: "".to_string()})
            }).withf(|url| url == "http://coordinator/collaboration/1/result_ids");
        let mut client = MockCsClient::new();
        client.expect_get_secret()
            .times(0);
        let res = result("http://coordinator",1, 1, &client, &net).await;
        assert_err!(res);
        net.checkpoint();
        client.checkpoint();
        Ok(())
    }
}