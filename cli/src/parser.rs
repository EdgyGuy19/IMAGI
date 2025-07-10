use serde::{Deserialize, Serialize};

pub struct Payload {
    user_id: String,
    readMe: String,
    source_files: Vect<SourceFile>,
    test_results: String,
}

pub struct SourceFile {
    filename: String,
    content: String,
}

#[tokio::main]

pub async fn main() -> Result<(), reqwest::Error> {
    let url = "http://localhost:8000/grade"; // may need replacement for the python api
    let response = reqwest::Client::new().post(url).send().await?;

    Ok(())
}
