use git2::Repository;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

//WE NEED ADD EXEPTION FOR THE TASK 13 BECAUSE OF NEWSFEED DIR

//IMPORTANT!!! ADD AS A COMMAND TO CLI!!!!
//Clones students' repos. Students' names from txt file and gets task number as input
// Also transform them into JSON format. Maybe find a better way to do it later??
//OBS! CLONES INTO CURRENT DIR!
// Should create a json with the paths to the directories of students(maybe)??
pub fn clone_repos(students: Vec<String>, task: String) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory
    let current_dir = std::env::current_dir()?;
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
    }
    Ok(())
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
        if file_path.is_file() && file_path.extension().map_or(false, |ext| ext == extension) {
            let file_name = file_path //maybe redo this cause it AI slop??
                .file_name()
                .unwrap() // safe if you know it's a file
                .to_string_lossy()
                .to_string();
            if !file_name.contains("Test") {
                files.push(file_path);
                names.push(file_name);
            }
        }
    }
    Ok((files, names))
}

pub fn parse_contents() {}

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

pub fn run_java_tests(
    students_repo: &Path,
    tests_dir: &Path,
    jars_dir: &Path,
) -> Result<String, Box<dyn std::error::Error>> {
    //1. Copy the tests from the directory and jars files
    let mut junit_jar;
    let mut hamcrest_jar;
    for file in fs::read_dir(tests_dir)? {
        let file = file?;
        let path = file.path();
        fs::copy(path, students_repo); //copy into src FIXXX
    }
    for file in fs::read_dir(jars_dir)? {
        let file = file?;
        let path = file.path();
        fs::copy(path, students_repo); //copy into src FIXXX
        let class_name = path.to_string_lossy();
        if class_name.contains("junit") {
            junit_jar = path;
        } else {
            hamcrest_jar = path;
        }
    }
    //2. Compile all the java files

    let test_classes = find_test_classes(students_repo.join("tests"))?; //add src here
    //3. Compile java files and run each test

    //4. Collect the results.

    //5. Return them or create json file
    Ok(())
}

//
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
