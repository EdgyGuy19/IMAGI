use crate::json_parser::SourceFile;
use crate::json_parser::create_feedback_json;
use crate::json_parser::create_payload_json;
use crate::json_parser::parse_source_file;
use reqwest::Client;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

//check single student??
//find research about grading with AI or sum like that
//ask about docs? Need to make another function and modifications? Ask students to always write in README instead?
//
//WE NEED ADD EXEPTION FOR THE TASK 13 BECAUSE OF NEWSFEED DIR OR ASK STUDENT TO REDO THE DIRS FROM NEWSFEED INTO SRC

//IMPORTANT!!! ADD AS A COMMAND TO CLI!!!!
//Clones students' repos. Students' names from txt file and gets task number as input
// Also transform them into JSON format. Maybe find a better way to do it later??
//OBS! CLONES INTO CURRENT DIR!
// Should create a json with the paths to the directories of students(maybe)??
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
    let base_url = "git@gits-15.sys.kth.se:inda-24/";
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
//Gets called when we transoform payload to JSON.
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

//IMPORTANT!!! ADD AS A COMMAND TO CLI!!!!
//Clones tests for tasks from inda-master org. OBS! CLONES INTO CURRET DIR!
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

    Ok(())
}

//somehow add readme here
pub fn create_payload(
    students_repo: PathBuf,
    path_to_task_dir: PathBuf,
    tests_dir: PathBuf,
    jars_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_string = fs::read_to_string(students_repo)?;
    let mut readme = String::new();
    let map: HashMap<String, PathBuf> = serde_json::from_str(&json_string)?;
    if let Some(val) = map.values().next() {
        let mut readme_path = val.clone(); // val: &PathBuf
        readme_path.pop(); // removes "src"
        readme_path.push("README.md");
        readme = std::fs::read_to_string(&readme_path)?;
    }
    let dir_path = path_to_task_dir.join("json_files");
    std::fs::create_dir_all(&dir_path)?;
    for (key, value) in &map {
        let mut source_files: Vec<SourceFile> = Vec::new();
        let (paths, names) = transform_contents(value)?;
        let test_results = run_java_tests(value.as_path(), &tests_dir, &jars_dir)?;
        for (name, path) in names.iter().zip(paths.iter()) {
            let source_file = parse_source_file(name, path)?;
            source_files.push(source_file);
        }
        let payload =
            create_payload_json(key.to_string(), readme.clone(), source_files, test_results)?;
        let json_path_name = format!("{}.json", key);
        let json_path = dir_path.join(json_path_name);
        std::fs::write(json_path, payload)?;
    }
    Ok(())
}

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

pub fn run_java_tests(
    students_src: &Path,
    tests_dir: &Path,
    jars_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Move any pre-existing student test files to student_tests/
    let mut test_files_to_move = Vec::new();

    for entry in fs::read_dir(students_src)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("Test") {
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
                if name.ends_with("Test.java") || name.ends_with("Tests.java") {
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

pub async fn send_payload(
    json_dir: PathBuf,
    output_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Start the API server in background

    fs::create_dir_all(&output_dir)?;
    let mut server = Command::new("uvicorn").arg("AI_api.api:app").spawn()?;

    // 2. Wait a moment to let the server boot
    wait_for_server_ready().await;

    let api = reqwest::Client::new();
    for file in fs::read_dir(json_dir)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            let content = fs::read_to_string(path)?; //basic post request
            let post = api
                .post("http://127.0.0.1:8000/grade") // http://127.0.0.1:8000/docs to check the server
                .header("Content-Type", "application/json")
                .body(content)
                .send()
                .await?;

            if post.status().is_success() {
                let feedback: serde_json::Value = post.json().await?;
                let student_id = feedback["student_id"].as_str().unwrap_or("");
                let status = feedback["status"].as_str().unwrap_or("");
                let ai_feedback = feedback["feedback"].as_str().unwrap_or("");
                let feedback_json = create_feedback_json(
                    student_id.to_string(),
                    status.to_string(),
                    ai_feedback.to_string(),
                )?;
                let json_path_name = format!("{}_feedback.json", student_id);
                let json_path = output_dir.join(json_path_name);
                std::fs::write(json_path, feedback_json)?;
            } else {
                let err_text = post.text().await?;
                eprintln!("Error: {}", err_text);
            }
        }
    }
    server.kill()?; // After grading is done
    Ok(())
}

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
