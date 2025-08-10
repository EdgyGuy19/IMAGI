AI-Grader
Grader to help TAs at KTH in courses DD1337 and DD1338.
Automates cloning student repositories, running Java tests, grading with AI, and posting feedback to GitHub.
Table of Contents
Features
Installation
Configuration
Usage
API Integration
Examples
Directory Structure
Troubleshooting
Contributing
License
Credits
Features
Clone student repositories and solution repositories from GitHub
Compile and test student Java assignments using JUnit/Hamcrest
Grade assignments using OpenAI via a Python FastAPI service
Post feedback to GitHub issues automatically
Print test results and AI-generated feedback in the terminal
Installation
System Requirements
Linux (tested on Arch Linux)
Rust (via rustup)
Python 3.9+
Java JDK (javac, java)
Git
Dependencies
Rust
Add these to your Cargo.toml:
code
Toml
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json", "blocking", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
Python
Install with pip (recommended in a virtual environment):
code
Sh
pip install fastapi uvicorn openai pydantic
External
JUnit (junit-4.12.jar)
Hamcrest (hamcrest-core-1.3.jar)
Place these in a directory (e.g., /home/inda-master/jars)
Install System Packages (Arch Linux)
code
Sh
sudo pacman -S python python-pip git jdk-openjdk rust
Configuration
Environment Variables
Set these before running the CLI:
code
Sh
export AI_GRADER_JARS_DIR=/path/to/jars
export GITHUB_TOKEN=your_github_token
export GRADER_OPENAI_API_KEY=your_openai_api_key
Input Files
students.txt: List of student usernames (one per line, no spaces)
code
Code
alice
bob
charlie
# Each line should contain a student kth_ID.
Usage
CLI Commands
clone - Clone student repositories and create a JSON file with src paths.
tests - Clone all solution repos from inda-master into a specified output directory.
java - Compile and test all student Java files, collect results, and create JSON payloads.
results - Print test results from JSON file(s) in a clear terminal format.
grade - Send JSON payloads to the Python AI API for grading and post feedback to GitHub.
feedback - Print AI-generated feedback from JSON file(s) in a clear terminal format.
Help Output
Run grader help to see all commands and options.
API Integration
The Rust CLI interacts with a Python FastAPI service for grading.
Start the API Server
code
Sh
uvicorn AI_api.api:app
The server should run at http://127.0.0.1:8000/grade.
Examples
code
Sh
# Clone student repos for a task
grader clone --students students.txt --task task-1 --output ./output

# Clone solution repos for all tasks
grader tests --output ./solutions

# Compile and test student Java files, create JSON payloads
grader java --json ./output/task-1/src_paths.json --output ./output/task-1/compiled --tests ./solutions/task-1/src --jars ./jars

# Print test results from JSON files
grader results --json ./output/task-1/compiled/json_files

# Grade assignments using the AI API and post feedback to GitHub
grader grade --json ./output/task-1/compiled/json_files --output ./feedback

# Print AI-generated feedback from JSON files
grader feedback --json ./feedback
Directory Structure
code
Code
AI-Grader/
├── src/
│   ├── main.rs
│   ├── github_api.rs
│   └── json_parser.rs
├── AI_api/
│   └── api.py
├── jars/                # JUnit/Hamcrest jars
├── students.txt         # List of student usernames
├── output/              # Output for cloned repos, results, etc.
├── feedback/            # AI-generated feedback JSON files
└── solutions/           # Cloned solution repos
Troubleshooting
Java compilation failed: Ensure JDK and JAR files are present and paths are correct.
API errors: Make sure the FastAPI server is running and the OpenAI API key is set.
GitHub issue creation fails: Check your GITHUB_TOKEN and repo permissions.
Missing dependencies: Double-check installation steps above.
Contributing
Pull requests are welcome!
Please follow Rust and Python code style conventions.
Open issues for bugs or feature requests.
License
MIT
Credits
Contributors: [Your Name], [Others]
Libraries: clap, serde, reqwest, tokio, fastapi, openai, pydantic
Inspired by grading workflows at KTH
