use serde::{self, Deserialize, Serialize};
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
//struct for getting status for issues
#[derive(Serialize, Deserialize)]
pub struct StatusIssue {
    pub studentid: String,
    pub status: String,
}
//struct for using github API to get issues statuses
#[derive(Serialize, Deserialize)]
pub struct IssueTitle {
    pub title: String,
    #[serde(skip_deserializing)]
    pub number: i64,
    #[serde(skip_deserializing)]
    pub state: String,
    #[serde(skip_deserializing)]
    pub body: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: Option<String>,
    #[serde(skip_deserializing)]
    pub updated_at: Option<String>,
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

    // Remove Java comments before sending to API
    let content_without_comments = remove_comments(&content_json);

    let source_file = SourceFile {
        filename: filename.to_string(),
        content: content_without_comments,
    };
    Ok(source_file)
}

// Removes Java comments from code
fn remove_comments(code: &str) -> String {
    let mut result = String::new();
    let mut chars = code.chars().peekable();
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next(); // consume the '/'
                in_block_comment = false;
                result.push(' '); // preserve spacing
            }
            continue;
        } else if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push(c); // keep the newline
            }
            continue;
        } else if in_string {
            result.push(c);
            if c == '\\' && chars.peek().is_some() {
                // Handle escape sequence
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == '"' {
                in_string = false;
            }
        } else {
            match c {
                '/' => {
                    if chars.peek() == Some(&'/') {
                        chars.next(); // consume the second '/'
                        in_line_comment = true;
                        result.push(' '); // preserve spacing
                    } else if chars.peek() == Some(&'*') {
                        chars.next(); // consume the '*'
                        in_block_comment = true;
                        result.push(' '); // preserve spacing
                    } else {
                        result.push(c);
                    }
                }
                '"' => {
                    result.push(c);
                    in_string = true;
                }
                _ => result.push(c),
            }
        }
    }

    result
}

pub fn parse_issue_status(
    student: &str,
    status: &str,
) -> Result<StatusIssue, Box<dyn std::error::Error>> {
    let status_issue = StatusIssue {
        studentid: student.to_string(),
        status: status.to_string(),
    };
    Ok(status_issue)
}
