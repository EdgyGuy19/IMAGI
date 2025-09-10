mod github_api;
mod json_parser;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::github_api::{
    check_issues, clone_repos, create_payload, get_tests, print_feedback, print_test_results,
    send_payload,
};

#[derive(Parser)]
#[command(
    name = "imagi",
    about = "🦀 IMAGI: Automate grading of student Java assignments.",
    long_about = "🦀 IMAGI - Assistant Moderated AI-Generated Insights\n\
    \n\
    Available commands:\n\
      help      - Show this help message.\n\
      clone     - Clone student repositories and optionally compile/test Java files.\n\
      tests     - Clone all solution repos from inda-master into a specified output directory.\n\
      results   - Print test results from JSON file(s) in a clear terminal format.\n\
      generate - Send JSON payloads to the Python AI API for grading and post feedback to GitHub.\n\
      feedback  - Print AI-generated feedback from JSON file(s) in a clear terminal format.\n\
      issues    - Check GitHub issues for students and display their status (PASS, FAIL, KOMP, KOMPLETTERING).\n\
    \n\
    USAGE EXAMPLES:\n\
      imagi help\n\
        Show this help message.\n\
    \n\
      imagi clone -s/--students <path-to-students.txt> -t/--task <task-number> -o/--output <output-dir> -u/--unittest <solutions-src-dir>\n\
        Clone all student repos for the specified task into <output-dir>/<task-number>, creates src_paths.json and compiles/tests Java files.\n\
        The unittest parameter specifies the directory containing test files for compilation and testing.\n\
        Example:\n\
          imagi clone -s /home/inda-25-students.txt -t task-5 -o /home/inda-25 -u /home/inda-master/task-5/src\n\
        Example students.txt file:\n\
          alice\n\
          bob\n\
          charlie\n\
          # Each line should contain a student kth_ID.\n\
    \n\
      imagi tests -o/--output <output-dir>\n\
        Clone all solution repos from inda-master for all tasks into <output-dir>.\n\
    \n\
      imagi results -j/--json <path-to-json-or-dir>\n\
        Print test results from a JSON file or directory in a readable format.\n\
    \n\
      imagi generate -j/--json <json-dir> -o/--output <output-dir> [-m/--model <openai|gemini>]\n\
        Send JSON payloads to the Python AI API for grading and post feedback to GitHub.\n\
        Default model is 'openai'. If using 'gemini', a Python virtual environment must be set up.\n\
    \n\
      imagi feedback -j/--json <path-to-feedback-json-or-dir>\n\
        Print AI-generated feedback from a JSON file or directory in a readable format.\n\
    \n\
      imagi issues -s/--students <path-to-students.txt> -t/--task <task>\n\
        Check GitHub issues for all students in a task and display their status (PASS, FAIL, KOMP, KOMPLETTERING).\n\
        Shows a formatted table with student names and their issue status with corresponding emojis.\n\
    \n\
    Notes:\n\
      - All commands that clone or generate files require an explicit --output directory.\n\
      - Output directories will be created automatically if they do not exist.\n\
      - The 'generate' command integrates with the Python AI API and posts feedback to GitHub issues.\n\
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
        #[arg(short = 'u', long = "unittest", required = true)]
        tests: PathBuf,
    },
    Tests {
        #[arg(short = 'o', long)]
        output: PathBuf,
    },
    Results {
        #[arg(short = 'j', long)]
        json: PathBuf,
    },
    Generate {
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
    Issues {
        #[arg(short = 's', long)]
        students: PathBuf,
        #[arg(short = 't', long)]
        task: String,
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
            tests,
        } => {
            // Clone repositories
            if let Err(e) = clone_repos(
                students.to_path_buf(),
                task.to_string(),
                output.to_path_buf(),
            ) {
                eprintln!("Error while cloning the repos or creating the json: {}", e);
                return;
            }

            // Compile and test Java files after cloning
            // Construct the path to the generated src_paths.json
            let repos_dir = output.join(&task);
            let json_path = repos_dir.join("src_paths.json");
            let compiled_output = repos_dir.join("compiled");

            // Create the output directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(&compiled_output) {
                eprintln!("Error creating output directory: {}", e);
                return;
            }

            // Compile and test Java files
            if let Err(e) = create_payload(
                json_path,
                compiled_output,
                tests.to_path_buf(),
            ) {
                eprintln!(
                    "Error while compiling or running the java tests: {}",
                    e
                );
            } else {
                println!("Successfully cloned repositories and compiled/tested Java files!");
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
        Commands::Generate {
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
        Commands::Issues { students, task } => {
            if let Err(e) = check_issues(students.to_path_buf(), task.to_string()).await {
                eprint!("Error while trying to get the issues statuses: {}", e);
            }
        }
    }
}
