mod common;

#[cfg(test)]
mod test {
    use std::io::Write;
    use crate::common::{self, DBTestContext, create_correct_collaboration};
    use poem::test::{TestForm, TestFormField};
    use reqwest::StatusCode;
    use tempfile::NamedTempFile;

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
}
