use crate::json_parser::IssueTitle;
use crate::json_parser::SourceFile;
use crate::json_parser::StatusIssue;
use crate::json_parser::create_feedback_json;
use crate::json_parser::create_issue;
use crate::json_parser::create_payload_json;
use crate::json_parser::parse_issue_status;
use crate::json_parser::parse_source_file;
use reqwest::Client;
use reqwest::header::AUTHORIZATION;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::header::USER_AGENT;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

//clones students repos and creates json files with paths to their src dirs.
pub fn clone_repos(
    students: PathBuf,
    task: String,
    output_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory
    let file = File::open(students)?;
    let buf = std::io::BufReader::new(file);
    let mut students_list = Vec::new();
    for line in buf.lines() {
        let student = line?.trim().to_string();
        if student.is_empty() || student.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        students_list.push(student);
    }
    let mut map: HashMap<String, PathBuf> = HashMap::new();
    // Create ./task directory
    let repos_dir = output_dir.join(&task);
    std::fs::create_dir_all(&repos_dir)?;
    let base_url = "git@gits-15.sys.kth.se:inda-25/";
    for student in students_list {
        // Build repo URL and destination directory
        let student_url = format!("{}{}-{}.git", base_url, student, task);
        let repo_name = format!("{}-{}", student, task);
        let student_dir = repos_dir.join(&repo_name);
        Command::new("git")
            .arg("clone")
            .arg(&student_url)
            .arg(&student_dir)
            .status()?;
        let src_dir = student_dir.join("src");
        map.insert(student, src_dir);
    }
    let json_string = serde_json::to_string_pretty(&map)?;
    let json_path = repos_dir.join("src_paths.json");
    std::fs::write(json_path, json_string)?;
    Ok(())
}

//Function to transform student's task/homework into format for JSON parsing.
//Gets called when we create payload for api.
pub fn transform_contents(
    repo_dir: &Path,
) -> Result<(Vec<PathBuf>, Vec<String>), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut names = Vec::new();
    for file in fs::read_dir(repo_dir)? {
        let file = file?;
        let file_path = file.path();
        if file_path.is_file() {
            if let Some(name) = file_path.file_name().and_then(|n| n.to_str()) {
                if !name.contains("Test") && name.contains("java") {
                    names.push(name.to_string());
                    files.push(file_path);
                }
            }
        }
    }
    Ok((files, names))
}

//Clones tests for tasks from inda-master org.
pub fn get_tests(output_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let basic_url = "git@gits-15.sys.kth.se:inda-master/{task}.git";

    for task_num in 1..=18 {
        let task = format!("task-{}", task_num);
        let repo_url = basic_url.replace("{task}", &task);
        let dest_dir = output_dir.join(&task);
        println!("Cloning {} into {:?}", repo_url, dest_dir);
        Command::new("git")
            .arg("clone")
            .arg(&repo_url)
            .arg(&dest_dir)
            .status()?;

        Command::new("git")
            .arg("-C")
            .arg(&dest_dir)
            .arg("checkout")
            .arg("-B")
            .arg("solutions")
            .arg("origin/solutions")
            .status()?;
    }

    let quicksort = "quicksort";
    let repo_url = basic_url.replace("{task}", &quicksort);
    let dest_dir = output_dir.join(&quicksort);
    println!("Cloning {} into {:?}", repo_url, dest_dir);
    Command::new("git")
        .arg("clone")
        .arg(&repo_url)
        .arg(&dest_dir)
        .status()?;

    Command::new("git")
        .arg("-C")
        .arg(&dest_dir)
        .arg("checkout")
        .arg("-B")
        .arg("solutions")
        .arg("origin/solutions")
        .status()?;

    Ok(())
}

//creating payload from repo
pub fn create_payload(
    students_repo: PathBuf,
    path_to_task_dir: PathBuf,
    tests_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = fs::read_to_string(students_repo)?;
    let mut readme = String::new();
    let mut task = String::new();
    let map: HashMap<String, PathBuf> = serde_json::from_str(&json_string)?;
    if let Some(val) = map.values().next() {
        let mut readme_path = val.clone(); // val: &PathBuf
        readme_path.pop(); // removes "src"
        readme_path.push("README.md");
        readme = std::fs::read_to_string(&readme_path)?;
        readme_path.pop(); // remove readme path
        readme_path.pop(); //get task number;
        task = readme_path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_string();
    }
    let dir_path = path_to_task_dir;
    std::fs::create_dir_all(&dir_path)?;
    for (key, value) in &map {
        let mut source_files: Vec<SourceFile> = Vec::new();
        let (paths, names) = transform_contents(value)?;
        let test_results = run_java_tests(value.as_path(), &tests_dir)?;
        for (name, path) in names.iter().zip(paths.iter()) {
            let source_file = parse_source_file(name, path)?;
            source_files.push(source_file);
        }
        let payload = create_payload_json(
            key.to_string(),
            task.clone(),
            readme.clone(),
            source_files,
            test_results,
        )?;
        let json_path_name = format!("{}.json", key);
        let json_path = dir_path.join(json_path_name);
        std::fs::write(json_path, payload)?;
    }
    Ok(())
}

//used to get names for test files
fn find_test_classes(students_repo: PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut test_names = Vec::new();
    for file in fs::read_dir(students_repo)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with("Test.java") || filename.ends_with("Tests.java") {
                    // Remove .java extension to get the class name
                    let class_name = filename.trim_end_matches(".java").to_string();
                    test_names.push(class_name);
                }
            }
        }
    }
    Ok(test_names)
}

//function that runs java commands, copies jars and test files to students repos, and moves their own made tests
pub fn run_java_tests(
    students_src: &Path,
    tests_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Move any pre-existing student test files to student_tests/
    let mut test_files_to_move = Vec::new();
    let jars_dir = env::var("IMAGI_JARS_DIR").expect("Set the IMAGI_JARS_DIR environment variable");

    for entry in fs::read_dir(students_src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("Test.java") {
                    test_files_to_move.push((path.clone(), name.to_string()));
                }
            }
        }
    }

    if !test_files_to_move.is_empty() {
        let student_tests_dir = students_src.join("student_tests");
        fs::create_dir_all(&student_tests_dir)?;
        for (path, name) in test_files_to_move {
            let dest = student_tests_dir.join(name);
            fs::rename(&path, &dest)?;
        }
    }

    // 2. Copy test files from tests_dir into students_src
    for entry in fs::read_dir(tests_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with("Test.java")
                    || name.ends_with("Tests.java")
                    || name.ends_with("test.go")
                    || name.ends_with("Test.go")
                    || name.ends_with("Test.class")
                    || name.ends_with("Tests.class")
                    || name.ends_with("test.class")
                {
                    let dest = students_src.join(name);
                    fs::copy(&path, &dest)?;
                }
            }
        }
    }

    // 3. Copy JAR files from jars_dir into students_src
    for entry in fs::read_dir(jars_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".jar") {
                    let dest = students_src.join(name);
                    fs::copy(&path, &dest)?;
                }
            }
        }
    }

    // 4. Compile all the java files
    let compile_status = Command::new("sh")
        .arg("-c")
        .arg("javac -cp '.:junit-4.12.jar:hamcrest-core-1.3.jar' *.java")
        .current_dir(students_src)
        .status()?;

    if !compile_status.success() {
        return Err("Java compilation failed".into());
    }

    // 5. Find test classes and run the tests
    let test_classes = find_test_classes(students_src.to_path_buf())?;
    let run = Command::new("java")
        .arg("-cp")
        .arg(".:junit-4.12.jar:hamcrest-core-1.3.jar")
        .arg("org.junit.runner.JUnitCore")
        .args(&test_classes)
        .current_dir(students_src)
        .output()?;

    // 6. Return test results (stdout + stderr)
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);

    Ok(format!("{}\n{}", stdout, stderr))
}

// Print only the test_results field from JSON file(s) with clear terminal output.
pub fn print_test_results(json_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    fn print_test_result_from_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let v: Value = serde_json::from_str(&data)?;
        let test_results = v
            .get("test_results")
            .and_then(|tr| tr.as_str())
            .unwrap_or("<no test_results field>");
        println!("\x1b[1;34mFile: {}\x1b[0m", path.display());
        println!("\x1b[1;32mTest Results:\x1b[0m\n{}", test_results.trim());
        Ok(())
    }

    if json_path.is_file() {
        print_test_result_from_file(&json_path)?;
    } else if json_path.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&json_path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        files.sort();
        for file in files {
            print_test_result_from_file(&file)?;
            println!("{}", "-".repeat(60));
        }
    } else {
        eprintln!("Path does not exist: {}", json_path.display());
    }
    Ok(())
}

// Print only the feedback fields from JSON file(s) with clear terminal output.
pub fn print_feedback(json_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    fn print_feedback_from_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let v: Value = serde_json::from_str(&data)?;
        let student_id = v
            .get("student_id")
            .and_then(|id| id.as_str())
            .unwrap_or("<no student_id>");
        let status = v
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("<no status>");
        let feedback = v
            .get("feedback")
            .and_then(|f| f.as_str())
            .unwrap_or("<no feedback>");
        println!("\x1b[1;34mFile: {}\x1b[0m", path.display());
        println!("\x1b[1;33mStudent ID:\x1b[0m {}", student_id);
        println!("\x1b[1;32mStatus:\x1b[0m {}", status);
        println!("\x1b[1;36mFeedback:\x1b[0m\n{}", feedback.trim());
        Ok(())
    }

    if json_path.is_file() {
        print_feedback_from_file(&json_path)?;
    } else if json_path.is_dir() {
        let mut files: Vec<_> = fs::read_dir(&json_path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().map(|e| e == "json").unwrap_or(false) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect();
        files.sort();
        for file in files {
            print_feedback_from_file(&file)?;
            println!("{}", "-".repeat(60));
        }
    } else {
        eprintln!("Path does not exist: {}", json_path.display());
    }
    Ok(())
}

//sending payload to the AI-api. Receiving back the graded feedback as well
//model parameter determines which AI model to use ("openai" or "gemini")
pub async fn send_payload(
    json_dir: PathBuf,
    output_dir: PathBuf,
    model: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create output directory
    fs::create_dir_all(&output_dir)?;

    // Define API endpoint and server command based on model choice
    let model_str = model.unwrap_or("openai");
    let api_endpoint = match model_str {
        "gemini" => "http://127.0.0.1:8000/imagi_gemini",
        _ => "http://127.0.0.1:8000/imagi_gpt", // Default to OpenAI
    };

    // Get project root from environment variable
    let project_root = PathBuf::from(
        env::var("IMAGI_ROOT")
            .map_err(|_| "IMAGI_ROOT environment variable not set. Please set it to the path containing the AI_api directory.")?
    );

    // Verify AI_api directory exists
    if !project_root.join("AI_api").exists() {
        return Err(format!(
            "AI_api directory not found in: {}. Please check your IMAGI_ROOT setting.",
            project_root.display()
        )
        .into());
    }

    // Start the API server with the appropriate module and venv if needed
    let mut server = if model_str == "gemini" {
        // Check if venv exists (required for Gemini API, especially on Arch Linux)
        let venv_python = project_root.join("AI_api/venv/bin/python");
        if !venv_python.exists() {
            return Err("Virtual environment not found for Gemini API. Please create it using:\n\npython -m venv venv\nsource venv/bin/activate\npip install google-genai fastapi uvicorn\n\nAlternatively, edit github_api.rs to use system Python if your distro supports it.".into());
        }

        // Use Python from venv to run the server with geminiAPI module
        Command::new(venv_python)
            .arg("-m")
            .arg("uvicorn")
            .arg("AI_api.geminiAPI:app")
            .current_dir(&project_root)
            .spawn()?

        // UNCOMMENT THIS SECTION AND COMMENT OUT THE ABOVE SECTION IF YOU DON'T NEED A VIRTUAL ENVIRONMENT
        //
        // On Arch Linux, pip packages cannot be installed system-wide, so we use venv.
        // On Ubuntu/Debian and other systems where pip allows global installs, you can use this instead.
        // Remember to install the required packages with: pip install google-genai fastapi uvicorn
        //
        // Command::new("python")
        //     .arg("-m")
        //     .arg("uvicorn")
        //     .arg("AI_api.geminiAPI:app")
        //     .current_dir(&project_root)
        //     .spawn()?
    } else {
        // Use Python from venv to run the server with gptAPI module
        let venv_python = project_root.join("AI_api/venv/bin/python");
        if !venv_python.exists() {
            return Err("Virtual environment not found for OpenAI API. Please create it using:\n\npython -m venv AI_api/venv\nsource AI_api/venv/bin/activate\npip install fastapi uvicorn openai pydantic\n\nAlternatively, edit github_api.rs to use system Python if your distro supports it.".into());
        }

        Command::new(venv_python)
            .arg("-m")
            .arg("uvicorn")
            .arg("AI_api.gptAPI:app")
            .current_dir(&project_root)
            .spawn()?
    };

    // 2. Wait a moment to let the server boot
    wait_for_server_ready().await;

    let api = reqwest::Client::new();
    for file in fs::read_dir(json_dir)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            let content = fs::read_to_string(path)?; //basic post request
            let post = api
                .post(api_endpoint) // Use model-specific endpoint (either /imagi_gpt or /imagi_gemini)
                .header("Content-Type", "application/json")
                .body(content)
                .send()
                .await?;

            if post.status().is_success() {
                let feedback: serde_json::Value = post.json().await?;
                let student_id = feedback["student_id"].as_str().unwrap_or("");
                let task = feedback["task"].as_str().unwrap_or("");
                let status = feedback["status"].as_str().unwrap_or("");
                let ai_feedback = feedback["feedback"].as_str().unwrap_or("");
                let feedback_json = create_feedback_json(
                    student_id.to_string(),
                    status.to_string(),
                    ai_feedback.to_string(),
                )?;
                let json_path_name = format!("{}_feedback.json", student_id);
                let json_path = output_dir.join(json_path_name);
                fs::write(&json_path, feedback_json)?;

                let mut response = String::new();

                // Print formatted output with colors and better layout
                println!("\n{}", "=".repeat(80));
                println!(
                    "\x1b[1;36m📝 FEEDBACK READY FOR: {}\x1b[0m",
                    student_id.to_uppercase()
                );
                println!("{}", "=".repeat(80));

                println!("\x1b[1;33m📋 Task:\x1b[0m {}", task);
                println!("\x1b[1;32m✅ Status:\x1b[0m {}", status);

                println!("\n\x1b[1;35m💬 Generated Feedback:\x1b[0m");
                println!("{}", "-".repeat(50));
                // Format feedback with proper line breaks and indentation
                for line in ai_feedback.lines() {
                    println!("  {}", line);
                }
                println!("{}", "-".repeat(50));

                println!(
                    "\n\x1b[1;34m🤔 Would you like to create a GitHub issue for this student?\x1b[0m"
                );
                println!(
                    "   \x1b[32m[y]\x1b[0m Yes, create issue   \x1b[31m[n]\x1b[0m No, save locally only"
                );
                print!("\x1b[1;37m➤ Your choice: \x1b[0m");
                use std::io::{self, Write};
                io::stdout().flush()?;

                io::stdin().read_line(&mut response)?;
                while response.trim() != "n" && response.trim() != "y" {
                    println!(
                        "\x1b[1;31m❌ Invalid input!\x1b[0m Please enter \x1b[32m'y'\x1b[0m or \x1b[31m'n'\x1b[0m:"
                    );
                    print!("\x1b[1;37m➤ Your choice: \x1b[0m");
                    io::stdout().flush()?;
                    response.clear();
                    io::stdin().read_line(&mut response)?;
                }
                if response.trim() == "y" {
                    // Ask for teacher feedback
                    println!(
                        "\n\x1b[1;34m🧑‍🏫 Would you like to add your own feedback before creating the issue?\x1b[0m"
                    );
                    println!(
                        "   \x1b[32m[y]\x1b[0m Yes, add my feedback   \x1b[31m[n]\x1b[0m No, use AI feedback only"
                    );
                    print!("\x1b[1;37m➤ Your choice: \x1b[0m");
                    io::stdout().flush()?;

                    let mut teacher_response = String::new();
                    io::stdin().read_line(&mut teacher_response)?;

                    let complete_feedback = if teacher_response.trim() == "y" {
                        // Get teacher's feedback
                        println!(
                            "\n\x1b[1;36m📝 Enter your feedback (type 'DONE' on a new line when finished):\x1b[0m"
                        );
                        let mut teacher_feedback = String::new();
                        let mut line = String::new();

                        loop {
                            line.clear();
                            io::stdin().read_line(&mut line)?;
                            if line.trim() == "DONE" {
                                break;
                            }
                            teacher_feedback.push_str(&line);
                        }

                        // Combine teacher's feedback with AI feedback
                        format!(
                            "👨‍🏫 **Teacher's note**:\n\n{}\n\n---\n\n🤖 **AI Suggestions** (optional improvements, not requirements):\n\n{}",
                            teacher_feedback.trim(),
                            ai_feedback
                        )
                    } else {
                        // Use only AI feedback but include AI suggestions header with disclaimer
                        format!(
                            "🤖 **AI Suggestions** (optional improvements, not requirements):\n\n{}\n\nNote: These suggestions are meant to help you learn and improve - they are not mandatory requirements that must be completed.",
                            ai_feedback
                        )
                    };

                    send_issue(
                        task.to_string(),
                        student_id.to_string(),
                        status.to_string(),
                        complete_feedback,
                    )
                    .await?;
                } else if response.trim() == "n" {
                    println!(
                        "\n\x1b[1;33m📁 Feedback saved locally for {}\x1b[0m",
                        student_id
                    );
                    println!(
                        "   \x1b[90m💾 Location: {}\x1b[0m",
                        &json_path.to_string_lossy()
                    );
                    println!("   \x1b[90m🚫 No GitHub issue will be created.\x1b[0m");
                }

                println!("{}\n", "=".repeat(80));
            } else {
                let err_text = post.text().await?;
                eprintln!("Error: {}", err_text);
            }
        }
    }
    server.kill()?; // After grading is done
    Ok(())
}

//function to create github issue with the AI feedback
async fn send_issue(
    task: String,
    student: String,
    status: String,
    feedback: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("GITHUB_TOKEN").expect("Set the GITHUB_TOKEN environment variable");
    let org = "inda-25";
    let repo = format!("{}-{}", student, task);
    let url = format!(
        "https://gits-15.sys.kth.se/api/v3/repos/{}/{}/issues",
        org, repo
    );

    let issue = create_issue(status, feedback);
    let mut headers = HeaderMap::new();

    headers.insert(USER_AGENT, HeaderValue::from_static("AI-Grader"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token))?,
    );

    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .headers(headers)
        .json(&issue)
        .send()
        .await?;

    if res.status().is_success() {
        println!("\n\x1b[1;32m✅ SUCCESS: GitHub issue created!\x1b[0m");
        println!("   \x1b[90m🔗 Issue posted to repository successfully\x1b[0m");
    } else {
        let error_msg = res.text().await?;
        println!("\n\x1b[1;31m❌ FAILED: Could not create GitHub issue\x1b[0m");
        println!("   \x1b[90m🔍 Error details: {}\x1b[0m", error_msg);
    }

    Ok(())
}

//function to wait for server response. called when when start up the server
async fn wait_for_server_ready() {
    let client = Client::new();
    for _ in 0..30 {
        // Try for up to 30 seconds
        match client.get("http://127.0.0.1:8000/docs").send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("Server is ready!");
                return;
            }
            _ => {
                println!("Waiting for server...");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    panic!("Server did not start in time!");
}

pub async fn check_issues(
    students: PathBuf,
    task: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = env::var("GITHUB_TOKEN").expect("Set the GITHUB_TOKEN environment variable");
    let file = File::open(students)?;
    let buf = std::io::BufReader::new(file);
    let mut students_list = Vec::new();
    let mut list_issues: Vec<StatusIssue> = Vec::new();
    for line in buf.lines() {
        let student = line?.trim().to_string();
        if student.is_empty() || student.starts_with('#') {
            continue; // Skip empty lines and comments
        }
        students_list.push(student);
    }
    let org = "inda-25";
    for student in students_list {
        let repo = format!("{}-{}", student, task);
        let url = format!(
            "https://gits-15.sys.kth.se/api/v3/repos/{}/{}/issues",
            org, repo
        );
        let client = reqwest::Client::new();
        let titles = ["PASS", "FAIL", "KOMP", "KOMPLETTERING"];
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("AI-Grader"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("token {}", token))?,
        );
        let response = client.get(&url).headers(headers).send().await?;

        if response.status().is_success() {
            let issues: Vec<IssueTitle> = response.json().await?;

            // Check if any issue has the title we're looking for
            let mut found_matching_title = false;
            for issue in issues {
                for title in titles {
                    // Convert both to uppercase for case-insensitive matching
                    let issue_upper = issue.title.to_uppercase();
                    let title_upper = title.to_uppercase();

                    // Check if the issue title contains our status keyword
                    if issue_upper.contains(&title_upper) {
                        let status = title.to_string(); // Use our known status, not the full title
                        let student_issue = parse_issue_status(&student, &status)?;
                        list_issues.push(student_issue);
                        found_matching_title = true;
                        break;
                    }
                }
            }

            // If no matching title found, add NULL status
            if !found_matching_title {
                let student_issue = parse_issue_status(&student, "NULL")?;
                list_issues.push(student_issue);
            }
        } else {
            // Handle API error
            return Err(format!("API error: {}", response.status()).into());
        }
    }

    // Print the formatted data table
    println!("\n{}", "=".repeat(60));
    println!("| {:<20} | {:<15} | {:<5} |", "STUDENT", "STATUS", "");
    println!("|{:-<22}|{:-<17}|{:-<7}|", "", "", "");

    for issue in &list_issues {
        let emoji = match issue.status.as_str() {
            "PASS" => "✅",
            "FAIL" => "❌",
            "KOMP" | "KOMPLETTERING" => "🔄",
            _ => "❓",
        };
        println!(
            "| {:<20} | {:<15} | {:<5} |",
            issue.studentid, issue.status, emoji
        );
    }
    println!("{}", "=".repeat(60));

    Ok(())
}

// pub fn golang_run(
//     students_src: &Path,
//     tests_dir: &Path,
// ) -> Result<String, Box<dyn std::error::Error>> {
//     let mut test_files_to_move = Vec::new();
//     for entry in fs::read_dir(students_src)? {
//         let entry = entry?;
//         let path = entry.path();
//         if path.is_file() {
//             if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
//                 if name.contains("Test.java") {
//                     test_files_to_move.push((path.clone(), name.to_string()));
//                 }
//             }
//         }
//     }
//     if !test_files_to_move.is_empty() {
//         let student_tests_dir = students_src.join("student_tests");
//         fs::create_dir_all(&student_tests_dir)?;
//         for (path, name) in test_files_to_move {
//             let dest = student_tests_dir.join(name);
//             fs::rename(&path, &dest)?;
//         }
//     }

//     for entry in fs::read_dir(tests_dir)? {
//         let entry = entry?;
//         let path = entry.path();
//         if path.is_file() {
//             if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
//                 if name.ends_with("Test.java")
//                     || name.ends_with("Tests.java")
//                     || name.ends_with("test.go")
//                     || name.ends_with("Test.go")
//                 {
//                     let dest = students_src.join(name);
//                     fs::copy(&path, &dest)?;
//                 }
//             }
//         }
//     }
//     let run = Command::new("sh")
//         .arg("-c")
//         .arg("go test")
//         .current_dir(students_src)
//         .output()?;

//     let stdout = String::from_utf8_lossy(&run.stdout);
//     let stderr = String::from_utf8_lossy(&run.stderr);

//     Ok(format!("{}\n{}", stdout, stderr))
// }
