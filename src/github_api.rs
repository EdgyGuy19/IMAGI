use crate::json_parser::SourceFile;
use crate::json_parser::create_payload_json;
use crate::json_parser::parse_source_file;
use git2::Repository;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

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
pub fn clone_repos(students: PathBuf, task: String) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory
    let current_dir = std::env::current_dir()?;
    let file = File::open(students)?;
    let buf = std::io::BufReader::new(file);
    let mut students_list = Vec::new();
    for line in buf.lines() {
        let student = line?;
        students_list.push(student);
    }
    let mut map: HashMap<String, PathBuf> = HashMap::new();
    // Create ./task directory
    let repos_dir = current_dir.join(&task);
    std::fs::create_dir_all(&repos_dir)?;
    let base_url = "https://gits-15.sys.kth.se/inda-24/";
    for student in students_list {
        // Build repo URL and destination directory
        let student_url = format!("{}{}-{}", base_url, student, task);
        match Repository::clone(&student_url, &repos_dir) {
            Ok(_) => {
                println!("Cloned {} to {:?}", student_url, repos_dir);
                let repo_name = format!("{}-{}", student, task);
                let student_dir = repos_dir.join(repo_name);
                let src_dir = student_dir.join("src");
                map.insert(student, src_dir);
            }
            Err(e) => {
                eprintln!("Failed to clone{}: {}", student_url, e);
                continue;
            }
        };
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
pub fn get_tests() -> Result<(), Box<dyn std::error::Error>> {
    let basic_url = "https://gits-15.sys.kth.se/inda-master/{task}/tree/solutions"; //change to ssh link later on
    let current_dir = std::env::current_dir()?;

    for task_num in 1..=18 {
        let task = format!("task-{}", task_num);
        let repo_url = basic_url.replace("{task}", &task);
        let dest_dir = current_dir.join(&task);
        println!("Cloning {} into {:?}", repo_url, dest_dir);
        match Repository::clone(&repo_url, &dest_dir) {
            Ok(_) => println!("Successfully cloned {}", repo_url),
            Err(e) => eprintln!("Failed to clone {}: {}", repo_url, e),
        }
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
            let filename = path.to_string_lossy();
            if filename.ends_with("Test.java") {
                let class_name = filename.trim_end_matches("java").to_string();
                test_names.push(class_name);
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
    //1. Copy the tests from the directory and jars files
    let mut junit_jar: Option<PathBuf> = None;
    let mut hamcrest_jar: Option<PathBuf> = None;
    let mut java_path = Vec::new();
    let mut test_path = Vec::new();

    for file in fs::read_dir(tests_dir)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            let filename = path.to_string_lossy();
            if filename.ends_with("Test.java") {
                test_path.push(path);
            }
        }
    }

    for file in fs::read_dir(jars_dir)? {
        let file = file?;
        let path = file.path();
        let class_name = path.to_string_lossy();
        if class_name.contains("junit") {
            junit_jar = Some(path);
        } else {
            hamcrest_jar = Some(path);
        }
    }
    for file in fs::read_dir(students_src)? {
        let file = file?;
        let path = file.path();
        if path.is_file() {
            let filename = path.to_string_lossy();
            if filename.ends_with(".java") && !filename.ends_with("Test.java") {
                java_path.push(path);
            }
        }
    }

    let compilation_dir_path = students_src.join("classes");
    std::fs::create_dir(&compilation_dir_path)?;
    let compilation_dir_name = &compilation_dir_path
        .to_str()
        .expect("Path is not valid UTF-8")
        .to_string();

    let junit_jar = junit_jar.expect("JUnit jar not found in jars_dir");
    let hamcrest_jar = hamcrest_jar.expect("Hamcrest jar not found in jars_dir");

    let junit_str = junit_jar
        .to_str()
        .expect("JUnit jar path is not valid UTF-8")
        .to_string();
    let hamcrest_str = hamcrest_jar
        .to_str()
        .expect("Hamcrest jar path is not valid UTF-8")
        .to_string();

    let jars_str = format!("{}:{}", junit_str, hamcrest_str);

    //2. Compile all the java files

    let mut java_args = vec![
        "-d".to_string(),
        compilation_dir_name.to_string(),
        "-cp".to_string(),
        jars_str,
    ];

    java_args.extend(java_path.iter().map(|p| p.to_string_lossy().into_owned()));
    java_args.extend(test_path.iter().map(|p| p.to_string_lossy().into_owned()));

    let _compile = Command::new("javac")
        .args(&java_args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()?;

    //3. Run the tests
    let test_classes = find_test_classes(tests_dir.to_path_buf())?;
    let jars_to_comp = format!("{}:{}:{}", compilation_dir_name, junit_str, hamcrest_str);
    let mut run_args = vec![
        "-cp".to_string(),
        jars_to_comp,
        "org.junit.runner.JUnitCore".to_string(),
    ];
    run_args.extend(test_classes.clone());

    let run = Command::new("java").args(&run_args).output()?;

    //4. Return test results
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    if !stdout.is_empty() {
        Ok(stdout.to_string())
    } else {
        Ok(stderr.to_string())
    }
}
