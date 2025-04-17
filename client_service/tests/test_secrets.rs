
mod common;

#[cfg(test)]
mod test {
    use std::io::Write;

    use poem::test::{TestForm, TestFormField};
    use reqwest::StatusCode;
    use tokio;
    use tempfile::NamedTempFile;

    use crate::common;
    
    #[tokio::test]
    async fn test_upload_secret() {
        let collab = common::setup_env().await;
        common::register_input_party(collab.id, 1).await;
        // Create a temporary file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"data\n22").unwrap();
        let file_path = temp_file.path().to_owned();
        println!("{}", file_path.to_str().expect("Unable to get file path"));

        let client = common::test_client();

        let file = tokio::fs::File::open(file_path.clone()).await.unwrap();
        let field = TestFormField::async_reader(tokio::io::BufReader::new(file))
            .filename("data_csv")
            .name("data_csv");
        let resp = client.post(format!("/secrets/{}/1", collab.id))
            .multipart(TestForm::new().field(field))
            .send()
            .await;
        resp.assert_status_is_ok();
        let body = resp.0.into_body().into_string().await;
        assert!(true, "response: {:?}", body);
    }

    #[tokio::test]
    async fn test_upload_secret_no_collab() {
        let collab = common::setup_env().await;
        // Create a temporary file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"data\n22").unwrap();
        let file_path = temp_file.path().to_owned();

        let client = common::test_client();

        let file = tokio::fs::File::open(file_path.clone()).await.unwrap();
        let field = TestFormField::async_reader(tokio::io::BufReader::new(file))
            .filename("data_csv")
            .name("data_csv");
        let resp = client.post(format!("/secrets/{}/1", collab.id + 315))
            .multipart(TestForm::new().field(field))
            .send()
            .await;
        resp.assert_status(StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_upload_secret_incorrect_type() {
        let collab = common::setup_env().await;
        // Create a temporary file with some content
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"data\nabcd").unwrap();
        let file_path = temp_file.path().to_owned();

        let client = common::test_client();

        let file = tokio::fs::File::open(file_path.clone()).await.unwrap();
        let field = TestFormField::async_reader(tokio::io::BufReader::new(file))
            .filename("data_csv")
            .name("data_csv");
        let resp = client.post(format!("/secrets/{}/1", collab.id))
            .multipart(TestForm::new().field(field))
            .send()
            .await;
        resp.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
    }
}