# AI-Grader

AI-Grader is a CLI tool that automates grading of Java assignments for KTH courses DD1337 and DD1338. It streamlines the workflow for TAs by handling repository cloning, test execution, AI-based grading, and feedback posting.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [API Integration](#api-integration)
- [Examples](#examples)
- [Directory Structure](#directory-structure)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)
- [License](#license)
- [Credits](#credits)

## Features

- Clone student repositories and solution repositories from GitHub
- Compile and test student Java assignments using JUnit/Hamcrest
- Grade assignments using OpenAI via a Python FastAPI service
- Post feedback to GitHub issues automatically
- Print test results and AI-generated feedback in the terminal

## Installation

### System Requirements

- Linux (tested on Arch Linux)
- Rust (via [rustup](https://rustup.rs/))
- Python 3.9+
- Java JDK (javac, java)
- Git

### Dependencies

#### Git

To use this tool, you must set up an SSH key for authenticating with the inda-organization on GitHub.

1. **Generate an SSH key:**
   Follow the official guide:
   [How to generate an SSH key](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent)

2. **Add your SSH key to GitHub:**
   See instructions here:
   [How to add an SSH key to your GitHub account](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/adding-a-new-ssh-key-to-your-github-account)

**Tip:**
Make sure your SSH key is added to your SSH agent and associated with your GitHub account before running any commands that clone repositories(Try to clone repositories repository manually beforehand to make sure it works. One from inda master and one inda-25).


#### Rust

The recommended way to install Rust is via [rustup](https://rustup.rs/):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions.
After installation, ensure Rust is available by running:

```sh
rustc --version
cargo --version
```

#### Python

**Install Python (3.9 or newer):**

On Linux (Arch example):
```sh
sudo pacman -S python python-pip
```

On Ubuntu/Debian:
```sh
sudo apt-get update
sudo apt-get install python3 python3-pip
```

Install with `pip` (in a virtual environment if pip is blocked like on arch linux(pain in the ass and not recommended)):

```sh
  python3 -m venv venv
  source venv/bin/activate
```

```sh
pip install fastapi uvicorn openai pydantic
```
**On arch use pacman packages instead(pip packages can no longer be installed to the system root):**

```sh
sudo pacman -Syu python-fastapi uvicorn python-openai python-pydantic
```

#### External

- JUnit (`junit-4.12.jar`)
- Hamcrest (`hamcrest-core-1.3.jar`)
- Place these in a directory (e.g., `/home/inda-master/jars`)
- There is also a directory [jars](jars) which contains hamcrest and junit jar that you can use instead.

### Build the CLI

To build the CLI tool, run:

```sh
cargo build        # for development
cargo build --release   # for optimized release build
```

#### Global installation

To install the grader CLI globally, run from the repository:

```sh
cargo install --path .
```

This will place the `grader` binary in `~/.cargo/bin` (make sure this directory is in your `PATH`).
After installation, you can run `grader` from any directory:

```sh
grader help
```

If the command does not work try to add this to your .bashrc`, `.zshrc`, or equivalent:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
```

Alternatively, you can build and copy manually:

```sh
cargo build --release
sudo cp target/release/grader /usr/local/bin/grader
```

**Note:**
- Your binary will be named `cli` (from `[package] name = "cli"`).
- If you want the command to be `grader`, rename your package in `Cargo.toml` to `grader` or copy the binary as shown above.

## Configuration

### Environment Variables

Guide for setting up environment variables: [How to set environment variables](https://www.twilio.com/en-us/blog/how-to-set-environment-variables-html)

### Getting your OpenAI Api Key
How to get your own OpenAI API key:[Guide for creating API Key](https://medium.com/@lorenzozar/how-to-get-your-own-openai-api-key-f4d44e60c327)

### Creating a GitHub Token

To create a GitHub token:

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens).
2. Click "Generate new token".
3. Select the required scopes (e.g., `repo`, `workflow`).
4. Copy and save your token securely.

For more details, see [GitHub Docs: Creating a personal access token](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token).
![GitHub Token Requirements](pics/Token.png)

### Set these before running the CLI:

```sh
export AI_GRADER_JARS_DIR=/path/to/jars/directory
export GITHUB_TOKEN=your_github_token
export GRADER_OPENAI_API_KEY=your_openai_api_key
```

### Input Files

To use the cli you will need to create a .txt file with kth ids of all the students in your group.
**IMPORTANT! Only 1 name per line in the file, no spaces, no @kth.se**

- `students.txt`: List of student usernames
    ```
    alice
    bob
    charlie
    # Each line should contain a student kth_ID.
    ```

## Usage

### CLI Commands

- `clone`     - Clone student repositories and create a JSON file with src paths.
- `tests`     - Clone all solution repos from inda-master into a specified output directory.
- `java`      - Compile and test all student Java files, collect results, and create JSON payloads.
- `results`   - Print test results from JSON file(s) in a clear terminal format.
- `grade`     - Send JSON payloads to the Python AI API for grading and post feedback to GitHub.
- `feedback`  - Print AI-generated feedback from JSON file(s) in a clear terminal format.

### Help Output

Run `grader help` to see all commands, options and how each command works.

## API Integration

The Rust CLI interacts with a Python FastAPI service for grading.

### Start the API Server

The server starts up automatically with the grade command and shuts down after the last student has been graded.

The server should run at `http://127.0.0.1:8000/grade`.

## Examples

```sh
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
```
**Note:**
When compiling and running tests, any student-written test files (e.g., `*Test.java`) are moved to a `student_tests/` directory to avoid conflicts with the provided tests.

## Directory Structure

```
AI-Grader/
├── src/
│   ├── main.rs          # Rust main file where we call all functions
│   ├── github_api.rs    # Rust file with all the functions that are used in the main.rs
│   └── json_parser.rs   # Rust file containing JSON formats and functions for formatting to JSON
├── AI_api/
│   └── api.py           # File for the AI grading API
├── jars/                # JUnit/Hamcrest jars
```

## Troubleshooting

- **Java compilation failed:** Ensure JDK and JAR files are present and paths are correct.
- **API errors:** Make sure the FastAPI server is running and the OpenAI API key is set.
- **GitHub issue creation fails:** Check your `GITHUB_TOKEN` and repo permissions.
- **Missing dependencies:** Double-check installation steps above.
- **Command not found:** Make sure you installed the CLI globally and your binary name matches (`cli` or `grader`). Check that `~/.cargo/bin` is in your `PATH`.
- **Missing environment variables:** Ensure you have set `GITHUB_TOKEN`, `AI_GRADER_JARS_DIR`, and `GRADER_OPENAI_API_KEY` before running the CLI.

- **If the error persists contact me via slack or create a github issue!!!:**

## Contributing

Pull requests are welcome!
Please format Rust code with `cargo fmt` and Python code with `black` before submitting pull requests.
Open issues for bugs or feature requests.

## License

MIT

## Credits

- Contributors: Edgar Palynski: [EdgyGuy19](https://github.com/EdgyGuy19)
- Libraries: clap, serde, reqwest, tokio, fastapi, openai, pydantic
- Inspired by hating repobee(shoutout)
