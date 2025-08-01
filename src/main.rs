mod github_api;
mod json_parser;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::github_api::{clone_repos, create_payload, get_tests, print_test_results, send_payload};

#[derive(Parser)]
#[command(
    name = "grader",
    about = "🦀 Rust Grader CLI: Automate grading of student Java assignments.",
    long_about = "🦀 Rust Grader CLI\n\
    \n\
    Available commands:\n\
      help    - Show this help message.\n\
      clone   - Clone student repositories into a specified output directory and create a JSON file with src paths.\n\
      tests   - Clone all solution repos from inda-master into a specified output directory.\n\
      java    - Compile and test all student Java files, collect results, and create JSON payloads.\n\
    \n\
    USAGE EXAMPLES:\n\
      grader help\n\
        Show this help message.\n\
    \n\
      grader clone --students <path-to-students.txt> --task <task-number> --output <output-dir>\n\
        Clone all student repos for the specified task into <output-dir>/<task-number> and create src_paths.json.\n\
        Example:\n\
          grader clone --students /home/inda-25-students.txt --task task-5 --output /home/inda-25/task-5\n\
        Example students.txt file:\n\
          alice\n\
          bob\n\
          charlie\n\
          # Each line should contain a student kth_ID.\n\
    \n\
      grader tests --output <output-dir>\n\
        Clone all solution repos from inda-master for all tasks into <output-dir>.\n\
    \n\
      grader java --json <src_paths.json> --output <output-dir> --tests <solutions-src-dir> --jars <jars-dir>\n\
        Compile and test all student Java files, collect results, and create JSON payloads for each student.\n\
        Example:\n\
          grader java --json /home/inda-25/task-5/src_paths.json \\\n\
                      --output /home/inda-25/task-5/compiled \\\n\
                      --tests /home/inda-master/task-5/src \\\n\
                      --jars /home/inda-master/jars\n\
    \n\
    Notes:\n\
      - All commands that clone or generate files require an explicit --output directory.\n\
      - Output directories will be created automatically if they do not exist.\n\
      - The 'grade' command is planned for future integration with the Python AI API and GitHub feedback.\n"
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
        #[arg(long)]
        output: PathBuf,
    }, //maybe add a parameter for output dir instead of cloning into current dir
    Tests {
        #[arg(long)]
        output: PathBuf,
    },
    Java {
        #[arg(long)]
        json: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        tests: PathBuf,
    },
    Results {
        #[arg(long)]
        json: PathBuf,
    },
    Grade {
        #[arg(long)]
        json: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Tests { output } => {
            if let Err(e) = get_tests(output.to_path_buf()) {
                eprintln!("Error getting the tests: {}", e);
            }
        }
        Commands::Clone {
            students,
            task,
            output,
        } => {
            if let Err(e) = clone_repos(
                students.to_path_buf(),
                task.to_string(),
                output.to_path_buf(),
            ) {
                eprintln!("Error while cloning the repos or creating the json: {}", e);
            }
        }
        Commands::Java {
            json,
            output,
            tests,
        } => {
            if let Err(e) = create_payload(
                json.to_path_buf(),
                output.to_path_buf(),
                tests.to_path_buf(),
            ) {
                eprintln!(
                    "Error while compiling or running the java tests or parsing the students' results: {}",
                    e
                );
            }
        }
        Commands::Results { json } => {
            if let Err(e) = print_test_results(json.to_path_buf()) {
                eprint!(
                    "Error while getting the json file/dir or while printing test results: {}",
                    e
                );
            }
        }
        Commands::Grade { json, output } => {
            if let Err(e) = send_payload(json.to_path_buf(), output.to_path_buf()).await {
                eprint!("Error while grading the students: {}", e);
            }
        }
    }
}
