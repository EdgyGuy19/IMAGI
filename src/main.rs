mod github_api;
mod json_parser;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::github_api::{
    clone_repos, create_payload, get_tests, print_feedback, print_test_results, send_payload,
};

#[derive(Parser)]
#[command(
    name = "grader",
    about = "🦀 Rust Grader CLI: Automate grading of student Java assignments.",
    long_about = "🦀 Rust Grader CLI\n\
    \n\
    Available commands:\n\
      help      - Show this help message.\n\
      clone     - Clone student repositories into a specified output directory and create a JSON file with src paths.\n\
      tests     - Clone all solution repos from inda-master into a specified output directory.\n\
      java      - Compile and test all student Java files, collect results, and create JSON payloads.\n\
      results   - Print test results from JSON file(s) in a clear terminal format.\n\
      grade     - Send JSON payloads to the Python AI API for grading and post feedback to GitHub.\n\
      feedback  - Print AI-generated feedback from JSON file(s) in a clear terminal format.\n\
    \n\
    USAGE EXAMPLES:\n\
      grader help\n\
        Show this help message.\n\
    \n\
      grader clone -s/--students <path-to-students.txt> -t/--task <task-number> -o/--output <output-dir>\n\
        Clone all student repos for the specified task into <output-dir>/<task-number> and create src_paths.json.\n\
        Example:\n\
          grader clone -s /home/inda-25-students.txt -t task-5 -o /home/inda-25/task-5\n\
        Example students.txt file:\n\
          alice\n\
          bob\n\
          charlie\n\
          # Each line should contain a student kth_ID.\n\
    \n\
      grader tests -o/--output <output-dir>\n\
        Clone all solution repos from inda-master for all tasks into <output-dir>.\n\
    \n\
      grader java -j/--json <src_paths.json> -o/--output <output-dir> -t/--tests <solutions-src-dir>\n\
        Compile and test all student Java files, collect results, and create JSON payloads for each student.\n\
        When compiling and running tests, any student-written test files (e.g., *Test.java) are moved to a student_tests/ directory to avoid conflicts with the provided tests.\n\
        Example:\n\
          grader java -j /home/inda-25/task-5/src_paths.json \\\n\
                      -o /home/inda-25/task-5/compiled \\\n\
                      -t /home/inda-master/task-5/src \\\n\
    \n\
      grader results -j/--json <path-to-json-or-dir>\n\
        Print test results from a JSON file or directory in a readable format.\n\
    \n\
      grader grade -j/--json <json-dir> -o/--output <output-dir> [-m/--model <openai|gemini>]\n\
        Send JSON payloads to the Python AI API for grading and post feedback to GitHub.\n\
        Default model is 'openai'. If using 'gemini', a Python virtual environment must be set up.\n\
    \n\
      grader feedback -j/--json <path-to-feedback-json-or-dir>\n\
        Print AI-generated feedback from a JSON file or directory in a readable format.\n\
    \n\
    Notes:\n\
      - All commands that clone or generate files require an explicit --output directory.\n\
      - Output directories will be created automatically if they do not exist.\n\
      - The 'grade' command integrates with the Python AI API and posts feedback to GitHub issues.\n\
      "
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Clone {
        #[arg(short = 's', long)]
        students: PathBuf,
        #[arg(short = 't', long)]
        task: String,
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    Tests {
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    Java {
        #[arg(short = 'j', long)]
        json: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(short = 't', long)]
        tests: PathBuf,
    },
    Results {
        #[arg(short = 'j', long)]
        json: PathBuf,
    },
    Grade {
        #[arg(short = 'j', long)]
        json: PathBuf,
        #[arg(short = 'o', long)]
        output: PathBuf,
        #[arg(short = 'm', long, default_value = "openai", value_parser = ["openai", "gemini"])]
        model: String,
    },
    Feedback {
        #[arg(short = 'j', long)]
        json: PathBuf,
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
        Commands::Grade {
            json,
            output,
            model,
        } => {
            if let Err(e) =
                send_payload(json.to_path_buf(), output.to_path_buf(), Some(model)).await
            {
                eprint!("Error while grading the students: {}", e);
            }
        }
        Commands::Feedback { json } => {
            if let Err(e) = print_feedback(json.to_path_buf()) {
                eprint!(
                    "Error while getting the json file/dir or while printing the feedback: {}",
                    e
                );
            }
        }
    }
}
