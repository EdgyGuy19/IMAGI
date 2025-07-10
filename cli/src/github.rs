use git2::Repository;
use std::fs;
use std::path::PathBuf;

pub fn clone_repos(students: Vec<String>, task: String) -> Result<(), Box<dyn std::error::Error>> {
    // Get current working directory
    let base_path: PathBuf = std::env::current_dir()?;
    // Create ./task directory
    let repos_dir: PathBuf = current_dir.join(&task);
    std::fs::create_dir_all(&repos_dir)?;
    let base_url: String = "https://gits-15.sys.kth.se/inda-24/";
    for student in students {
        // Build repo URL and destination directory
        let student_url = format!("{}{student}-{task}", base_url);
        let student_dir: PathBuf = repos_dir.join(student);
        std::fs::create_dir_all(&student_dir)?;
        match Repository::clone(&student_url, &student_dir) {
            Ok(_) => println!("Cloned {} to {:?}", student_url, student_dir),
            Err(e) => eprintln!("Failed to clone{}: {}", student_url, e),
        };
    }
    //Parse repos to JSON
    Ok(());
}
