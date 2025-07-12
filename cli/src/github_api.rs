use crate::json_parser;
use crate::json_parser::parse_payload;
use crate::json_parser::SourceFile;
use git2::{Repository, build::RepoBuilder};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::fs::copy;
use std::process::Command;
use std::fs::read_dir;

//IMPORTANT!!! ADD AS A COMMAND TO CLI!!!!
//Clones students' repos. Students' names from txt file and gets task number as input
// Also transform them into JSON format. Maybe find a better way to do it later??
//OBS! CLONES INTO CURRENT DIR!
pub fn clone_repos(students: Vec<String>, task: String) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory
    let base_path = std::env::current_dir()?;
    // Create ./task directory
    let repos_dir = current_dir.join(&task);
    std::fs::create_dir_all(&repos_dir)?;
    let base_url = "https://gits-15.sys.kth.se/inda-24/"; // change to ssh link later on
    for student in students {
        // Build repo URL and destination directory
        let student_url = format!("{}{student}-{task}", base_url);
        let student_dir = repos_dir.join(student);
        std::fs::create_dir_all(&student_dir)?;
        match Repository::clone(&student_url, &student_dir) {
            Ok(_) => println!("Cloned {} to {:?}", student_url, student_dir),
            Err(e) => eprintln!("Failed to clone{}: {}", student_url, e),
        };
        let src_dir = student_dir.join("src");
        let (all_file_paths, all_file_names) = transform_contents(src_dir, "java")?;
        let mut all_source_files = Vec<SourceFile>::new();
        for (file_path, file_name) in all_file_paths.iter().zip(all_file_names.iter()) {
            match json_parser::parse_source_file(file_name, file_path) {
                Ok(source_file) => all_source_files.push(source_file),
                Err(e) => eprintln!("Failed to parse {}: {}", file_name, e),
            }
        }
        //Run java tests here
        let test_result = "True";
        let read_me_path = student_dir.join("README.md");
        let json_payload = parse_payload(student, read_me_path, all_source_files, test_results);
    }
    Ok(());
}

//Function to transform student's task/homework into format for JSON parsing.
//Gets called when we transoform payload to JSON.
pub fn transform_contents(
    repo_dir: &Path,
    extension: &str,
) -> Result<(Vec<PathBuf>, Vec<String>), Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let mut names = Vec::new();
    for file in fs::read_dir(repo_dir)? {
        let file = file?;
        let file_path = file.path();
        if file_path.is_file() && file_path.extension().map_or(false, |ext| ext = extension) {
            let file_name = file_path
                .file_name()
                .unwrap() // safe if you know it's a file
                .to_string_lossy()
                .to_string();
            if (!file_name.contains("Test")) {
                files.push(file_path);
                names.push(file_name);
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

pub fn run_java_tests(students_repo: &Path, tests_dir: &Path)-> Result<String, Box<dyn std::error::Error>> {

}
