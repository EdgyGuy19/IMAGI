use serde::{Deserialize, Serialize};
use std::path::Path;

//struct for payload that we use for AI api later
#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub user_id: String,
    pub read_me: String,
    pub source_files: Vec<SourceFile>,
    pub test_results: String,
}

//struct for students' files
#[derive(Serialize, Deserialize)]
pub struct SourceFile {
    filename: String,
    content: String,
}

pub fn parse_payload(
    user_id: &str,
    read_me: &Path,
    source_files: Vec<SourceFile>,
    test_results: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Build Payload struct
    let read_me_json = std::fs::read_to_string(read_me)?;
    let test_result_json = std::fs::read_to_string(test_results)?;

    let payload = Payload {
        user_id: user_id.to_string(),
        read_me: read_me_json,
        source_files,
        test_results: test_result_json,
    };
    // 2. Serialize to JSON string
    let json_string = serde_json::to_string(&payload)?;
    // 3. Return JSON string
    Ok(json_string)
}

pub fn parse_source_file(
    filename: &str,
    content: &Path,
) -> Result<SourceFile, Box<dyn std::error::Error>> {
    let content_json = std::fs::read_to_string(content)?;
    let source_file = SourceFile {
        filename: filename.to_string(),
        content: content_json,
    };
    Ok(source_file)
}
