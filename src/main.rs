mod github_api;
mod json_parser;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::github_api::{clone_repos, create_payload, get_tests};

#[derive(Parser)]
#[command(
    name = "grader",
    about = "🦀 Rust Grader CLI: Automate grading of student Java assignments.",
    long_about = "🦀 Rust Grader CLI\n\nAvailable commands:\n  help    - Show this help message.\n  clone   - Clone student repositories and create a JSON file with src paths into the current dir.\n  tests   - Clone all solution repos from inda-master into the current directory.\n  java    - Compile and test all student Java files, collect results, and create JSON payloads.\n\nUSAGE EXAMPLES:\n  grader help\n    Show this help message.\n  grader clone --students <path-to-students.txt> --task <task-number>\n    Clone all student repos for the specified task and create src_paths.json.\n    Example:\n      grader clone --students /home/inda-25-students.txt --task task-5\n    Example students.txt file:\n      alice\n      bob\n      charlie\n      # Each line should contain a student kth_ID.\n  grader tests\n    Clone all solution repos from inda-master for all tasks into the current directory.\n  grader java --json_paths <src_paths.json> --output_dir <output-dir> --tests_dir <solutions-src-dir> --jars_dir <jars-dir>\n    Compile and test all student Java files, collect results, and create JSON payloads for each student.\n    Example:\n      grader java --json_paths /home/inda-25/task-5/src_paths.json \\\n                  --output_dir /home/inda-25/task-5/compiled \\\n                  --tests_dir /home/inda-master/task-5/src \\\n                  --jars_dir /home/inda-master/jars\nNotes:\n  - All commands operate in the current working directory unless specified otherwise.\n  - Output directories will be created automatically if they do not exist.\n  - The 'grade' command is planned for future integration with the Python AI API and GitHub feedback.\n"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clone {
        #[arg(long)]
        students: PathBuf,
        #[arg(long)]
        task: String,
    }, //maybe add a parameter for output dir instead of cloning into current dir
    Tests {},
    Java {
        #[arg(long)]
        json_paths: PathBuf,
        #[arg(long)]
        output_dir: PathBuf,
        #[arg(long)]
        tests_dir: PathBuf,
        #[arg(long)]
        jars_dir: PathBuf,
    },
    //Grade {json_dir: PathBuf, github_token: String}, idk about that one yet
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tests {} => {
            if let Err(e) = get_tests() {
                eprintln!("Error getting the tests: {}", e);
            }
        }

        Commands::Clone { students, task } => {
            if let Err(e) = clone_repos(students.to_path_buf(), task.to_string()) {
                eprintln!("Error while cloning the repos or creating the json: {}", e);
            }
        }

        Commands::Java {
            json_paths,
            output_dir,
            tests_dir,
            jars_dir,
        } => {
            if let Err(e) = create_payload(
                json_paths.to_path_buf(),
                output_dir.to_path_buf(),
                tests_dir.to_path_buf(),
                jars_dir.to_path_buf(),
            ) {
                eprintln!(
                    "Error while compiling or running the java tests or parsing the students' results: {}",
                    e
                );
            }
        }
    }
}
