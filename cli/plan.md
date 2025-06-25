[🦀 Rust Grader CLI]
 ├── Clone student repo
 ├── Parse README as task description
 ├── Run premade tests (e.g., `javac`, `java`)
 ├── Read code files (filter: .java)
 ├── Build full grading payload:
 │     - Test results
 │     - Task description (README)
 │     - Codebase summary (or raw code)
 ├── Send to Python LLM API
 └── Post returned feedback to GitHub issue

[🐍 Python AI Service]
 ├── Accepts POST with:
 │     - README
 │     - Source code
 │     - Test results
 ├── Uses OpenAI/Claude to:
 │     - Compare implementation to instructions
 │     - Identify missing features / violations
 │     - Suggest improvements or praise
 └── Sends JSON feedback back to Rust
