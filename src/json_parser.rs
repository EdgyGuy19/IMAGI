use serde::{Deserialize, Serialize};
use std::path::Path;

//struct for payload that we use for AI api later
#[derive(Serialize, Deserialize)]
pub struct Payload {
    user_id: String,
    task: String,
    read_me: String,
    source_files: Vec<SourceFile>,
    test_results: String,
}

//struct for students' files
#[derive(Serialize, Deserialize)]
pub struct SourceFile {
    filename: String,
    content: String,
}

//struct for ai grading
#[derive(Serialize, Deserialize, Debug)]
pub struct FeedbackEntry {
    student_id: String,
    status: String,
    feedback: String,
}

//struct for sending github issue
#[derive(Serialize, Deserialize)]
pub struct Issue {
    title: String,
    body: String,
}

//Functions for creating our structs and parsing to JSON

pub fn create_issue(title: String, body: String) -> Issue {
    Issue { title, body }
}

pub fn create_feedback_json(
    student_id: String,
    status: String,
    feedback: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let feedback = FeedbackEntry {
        student_id,
        status,
        feedback,
    };

    let json_string = serde_json::to_string(&feedback)?;

    Ok(json_string)
}

pub fn create_payload_json(
    user_id: String,
    task: String,
    read_me: String,
    source_files: Vec<SourceFile>,
    test_results: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let payload = Payload {
        user_id,
        task,
        read_me,
        source_files,
        test_results,
    };

    let json_string = serde_json::to_string(&payload)?;

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
