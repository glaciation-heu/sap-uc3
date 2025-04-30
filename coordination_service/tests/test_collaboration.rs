mod common;

#[cfg(test)]
mod test {
    use std::{io::Write, str::FromStr};
    use crate::common::{self, DBTestContext, create_correct_collaboration};
    use claim::assert_some;
    use coordination_service::db::models::Participation;
    use poem::test::{TestForm, TestFormField};
    use reqwest::StatusCode;
    use tempfile::NamedTempFile;
    use tokio_test::assert_ok;
    use uuid::Uuid;

    #[tokio::test]
    async fn create_collaboration_wrong_input() {

        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);

        // Create a temporary file with some content
        let mut tmp_program = NamedTempFile::new().unwrap();
        let mut tmp_config = NamedTempFile::new().unwrap();

        tmp_program.write_all(b"this is just some data").unwrap();
        tmp_config.write_all(
        br#"{
        "noSslValidation":true,
        "providers":[
            {"amphoraServiceUrl":"http://csmock/0/amphora",
            "baseUrl":"http://csmock/0/",
            "castorServiceUrl":"http://csmock/0/castor",
            "ephemeralServiceUrl":"http://csmock/0/",
            "id":1},
            {"amphoraServiceUrl":"http://csmock/1/amphora",
            "baseUrl":"http://csmock/1/",
            "castorServiceUrl":"http://csmock/1/castor",
            "ephemeralServiceUrl":"http://csmock/1/",
            "id":2}],
        "r":"141515903391459779531506841503331516415",
        "rinv":"133854242216446749056083838363708373830"}"#
        ).unwrap();

        let program_file = tokio::fs::File::open(tmp_program.path().to_owned()).await.unwrap();
        let config_file = tokio::fs::File::open(tmp_config.path().to_owned()).await.unwrap();

        let program_field = TestFormField::async_reader(tokio::io::BufReader::new(program_file))
            .filename("mpc_program")
            .name("mpc_program");
        let config_field = TestFormField::async_reader(tokio::io::BufReader::new(config_file))
            .filename("cs_config")
            .name("cs_config");
        let resp = client.post("/collaboration")
            .multipart(TestForm::new()
                .field(TestFormField::text("demo").name("name"))
                .field(TestFormField::text("data").name("csv_header_line"))
                .field(TestFormField::text("1").name("number_of_parties"))
                .field(program_field)
                .field(config_field)
            ).send().await;
        resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_collaboration() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);

        let resp = create_correct_collaboration(&client).await;
        // let body = resp.0.into_body().into_string().await;
        // assert!(false, "response: {:?}", body);
        resp.assert_status_is_ok();
        let json = resp.json().await;
        let resp_object = json.value().object();

        // test some parameters
        resp_object.assert_len(7);
        resp_object.get_opt("id").expect("id not found");
        resp_object.get_opt("name").expect("name not found").assert_string("demo");
        resp_object.get_opt("participation_number").expect("name not found").assert_i64(1);
    }

    #[tokio::test]
    async fn delete_collaboration() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let collab_resp = create_correct_collaboration(&client).await;
        collab_resp.assert_status_is_ok();
        let resp = collab_resp.json().await;
        let id = resp.value().object().get("id").i64();
        let resp = client.delete(format!("/collaboration/{}", id))
            .send().await;
        resp.assert_status_is_ok();
    }

    #[tokio::test]
    async fn delete_collaboration_no_collab() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);

        let resp = client.delete(format!("/collaboration/{}", 255))
            .send().await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn register_input_party() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let collab_resp = create_correct_collaboration(&client).await;
        collab_resp.assert_status_is_ok();
        let resp = collab_resp.json().await;
        let id = resp.value().object().get("id").i64();

        let resp = client.post(format!("/collaboration/{}/register-input-party/1", id))
            .send().await;
        resp.assert_status_is_ok();
    }

    #[tokio::test]
    async fn register_input_party_no_collaboration() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let resp = client.post("/collaboration/2/register-input-party/1")
            .send().await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn register_output_party() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);

        let collab_resp = create_correct_collaboration(&client).await;
        collab_resp.assert_status_is_ok();
        let resp = collab_resp.json().await;
        let id = resp.value().object().get("id").i64();

        let resp = client.post(format!("/collaboration/{}/register-output-party/1?party_client_endpoint=abc123", id))
            .send().await;
        resp.assert_status_is_ok();

    }
    #[tokio::test]
    async fn register_output_party_no_collaboration() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let resp = client.post("/collaboration/2/register-output-party/1?party_client_endpoint=abc123")
            .send().await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn register_secret_upload() {
        // Setup
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let collab_resp = create_correct_collaboration(&client).await;
        collab_resp.assert_status_is_ok();
        let resp = collab_resp.json().await;
        let id = resp.value().object().get("id").i64();

        // Register input party
        let resp = client.post(format!("/collaboration/{}/register-input-party/1", id))
            .send().await;
        resp.assert_status_is_ok();

        // Generate result UUID and register it
        let res_uuid = Uuid::new_v4();
        let res_resp = client.post(format!("/collaboration/{}/confirm-upload/1", id))
            .body_json(&vec![res_uuid.to_string()])
            .send().await;
        res_resp.assert_status_is_ok();

        // Check if correct UUID was registered
        let get_participation = client.get(format!("/collaboration/{}/input-parties", id))
            .send().await;
        get_participation.assert_status_is_ok();
        let body = get_participation.0.into_body().into_json::<Vec<Participation>>().await;
        let body = assert_ok!(body);
        assert_eq!(body.len(), 1, "The len of participations for the test collaboration is {} not 1", body.len());
        let participation = &body[0];
        if let Some(registered) = participation.secret_ids.clone() {
            assert_eq!(registered.len(), 1, "The number of registered parties is not correct. Exptected 1, got {}", registered.len());
            let uuid = assert_some!(registered[0].clone(), "Expected registered UUID to be Some(String).");
            let uuid = assert_ok!(Uuid::from_str(&uuid), "Unable to parse UUID from registered.");
            assert_eq!(uuid, res_uuid, "The result UUID is not equal to the registered one.");
        } else {
            assert!(false, "Party is not registered");
        }
    }
    #[tokio::test]
    async fn register_secret_upload_not_registered() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);
        let collab_resp = create_correct_collaboration(&client).await;
        collab_resp.assert_status_is_ok();
        let resp = collab_resp.json().await;
        let id = resp.value().object().get("id").i64();

        // Generate result UUID and register it
        let res_uuid = Uuid::new_v4();
        let res_resp = client.post(format!("/collaboration/{}/confirm-upload/1", id))
            .body_json(&vec![res_uuid.to_string()])
            .send().await;
        res_resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn register_secret_upload_no_collab() {
        let db = DBTestContext::new();
        let client = common::test_client(&db.db_url);

        // Generate result UUID and register it
        let res_uuid = Uuid::new_v4();
        let res_resp = client.post(format!("/collaboration/{}/confirm-upload/1", 1))
            .body_json(&vec![res_uuid.to_string()])
            .send().await;
        res_resp.assert_status(StatusCode::NOT_FOUND);
    }
}
