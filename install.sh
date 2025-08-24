#!/bin/bash

set -e  # Exit on error

echo "AI-Grader Installation Script"
echo "============================="

# Detect OS and package manager
detect_environment() {
    # Detect OS type
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Detect specific Linux distribution and package manager
        if [ -f /etc/arch-release ]; then
            OS="arch"
            PKG_MANAGER="pacman"
        elif [ -f /etc/fedora-release ]; then
            OS="fedora"
            PKG_MANAGER="dnf"
        elif [ -f /etc/gentoo-release ]; then
            OS="gentoo"
            PKG_MANAGER="emerge"
        elif [ -f /etc/SuSE-release ] || [ -f /etc/opensuse-release ]; then
            OS="suse"
            PKG_MANAGER="zypper"
        elif [ -f /etc/debian_version ]; then
            # This covers Debian, Ubuntu, and derivatives
            OS="debian"
            PKG_MANAGER="apt-get"
        elif [ -f /etc/alpine-release ]; then
            OS="alpine"
            PKG_MANAGER="apk"
        else
            OS="unknown_linux"
            PKG_MANAGER="unknown"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="mac"
        PKG_MANAGER="brew"
        # Check if Homebrew is installed
        if ! command -v brew &> /dev/null; then
            echo "Homebrew not found. Please install Homebrew first: https://brew.sh/"
            exit 1
        fi
    else
        echo "Unsupported operating system: $OSTYPE"
        exit 1
    fi

    echo "Detected OS: $OS"
    echo "Package manager: $PKG_MANAGER"
}

# Handle unknown distribution with interactive prompt
handle_unknown_distribution() {
    if [[ "$PKG_MANAGER" == "unknown" ]]; then
        echo "Could not determine your Linux distribution's package manager."
        echo "Please select your package manager:"
        echo "1) apt-get (Debian, Ubuntu, etc.)"
        echo "2) dnf (Fedora, RHEL, etc.)"
        echo "3) pacman (Arch, Manjaro, etc.)"
        echo "4) zypper (openSUSE)"
        echo "5) emerge (Gentoo)"
        echo "6) apk (Alpine)"
        echo "7) Other/manual installation"

        read -p "Enter selection (1-7): " selection

        case $selection in
            1) PKG_MANAGER="apt-get" ;;
            2) PKG_MANAGER="dnf" ;;
            3) PKG_MANAGER="pacman" ;;
            4) PKG_MANAGER="zypper" ;;
            5) PKG_MANAGER="emerge" ;;
            6) PKG_MANAGER="apk" ;;
            7)
                echo "You've selected manual installation."
                echo "Please install the required dependencies listed above."
                exit 0
                ;;
            *)
                echo "Invalid selection. Exiting."
                exit 1
                ;;
        esac
    fi
}

# Install base system dependencies (not Python packages)
install_system_dependencies() {
    echo "Installing system dependencies..."

    case $PKG_MANAGER in
        "pacman")
            echo "Installing for Arch Linux..."
            sudo pacman -Syu --needed --noconfirm python python-pip rustup git jdk-openjdk python-virtualenv
            ;;
        "dnf")
            echo "Installing for Fedora..."
            sudo dnf install -y python3 python3-pip rust cargo git java-latest-openjdk python3-virtualenv
            ;;
        "emerge")
            echo "Installing for Gentoo..."
            sudo emerge --ask dev-lang/python dev-python/pip dev-lang/rust dev-vcs/git virtual/jdk dev-python/virtualenv
            ;;
        "zypper")
            echo "Installing for openSUSE..."
            sudo zypper install -y python3 python3-pip rust cargo git java-latest-openjdk python3-virtualenv
            ;;
        "apt-get")
            echo "Installing for Debian/Ubuntu..."
            sudo apt-get update
            sudo apt-get install -y python3 python3-pip python3-venv curl git default-jdk
            # Install Rust if not available
            if ! command -v rustc &> /dev/null; then
                curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
                source $HOME/.cargo/env
            fi
            ;;
        "apk")
            echo "Installing for Alpine Linux..."
            sudo apk add python3 py3-pip rust cargo git openjdk11 py3-virtualenv
            ;;
        "brew")
            echo "Installing for macOS..."
            brew update
            brew install python rust openjdk git
            ;;
        *)
            echo "Unsupported package manager: $PKG_MANAGER"
            echo "Please install the following dependencies manually:"
            echo "- Python 3 and pip"
            echo "- Python virtualenv"
            echo "- Rust and Cargo"
            echo "- Git"
            echo "- Java JDK"
            exit 1
            ;;
    esac
}

# Set up Python virtual environment
setup_virtual_environment() {
    echo "Setting up Python virtual environment..."

    # Create virtual environment if it doesn't exist
    if [ ! -d "venv" ]; then
        python3 -m venv venv || python -m venv venv
    fi

    # Activate virtual environment
    source venv/bin/activate

    # Upgrade pip
    pip install --upgrade pip

    # Install Python dependencies
    echo "Installing Python packages in virtual environment..."
    pip install fastapi uvicorn openai pydantic google-generativeai

    # Deactivate virtual environment
    deactivate
}

# Update the github_api.rs file to use the venv for all Python API calls
update_github_api_rs() {
    echo "Updating github_api.rs to use the virtual environment..."

    # Backup the original file
    cp src/github_api.rs src/github_api.rs.bak

    # Use sed to update the file (commands will vary slightly for macOS vs Linux)
    if [[ "$OS" == "mac" ]]; then
        # macOS version of sed
        sed -i '' 's|Command::new("python")|Command::new("./venv/bin/python")|g' src/github_api.rs
        sed -i '' 's|Command::new(project_root.join("AI_api/venv/bin/python"))|Command::new(project_root.join("venv/bin/python"))|g' src/github_api.rs
    else
        # GNU/Linux version of sed
        sed -i 's|Command::new("python")|Command::new("./venv/bin/python")|g' src/github_api.rs
        sed -i 's|Command::new(project_root.join("AI_api/venv/bin/python"))|Command::new(project_root.join("venv/bin/python"))|g' src/github_api.rs
    fi

    echo "github_api.rs updated to use the virtual environment."
}

# Create startup script to activate environment and set variables
create_startup_script() {
    echo "Creating startup script..."

    cat > start_ai_grader.sh << EOL
#!/bin/bash
# AI-Grader startup script

# Activate Python virtual environment
source venv/bin/activate

# Set environment variables
export AI_GRADER_ROOT=$(pwd)
export AI_GRADER_JARS_DIR=$(pwd)/jars

# Check for required environment variables
if [ -z "\$GITHUB_TOKEN" ]; then
    echo "⚠️ GITHUB_TOKEN environment variable not set"
    echo "Please set it with: export GITHUB_TOKEN=your_github_token"
fi

if [ -z "\$GRADER_OPENAI_API_KEY" ] && [ -z "\$GRADER_GEMINI_API_KEY" ]; then
    echo "⚠️ No API keys set for AI models"
    echo "Set at least one of these:"
    echo "export GRADER_OPENAI_API_KEY=your_openai_api_key"
    echo "export GRADER_GEMINI_API_KEY=your_gemini_api_key"
fi

echo "AI-Grader environment ready!"
echo "Run commands with: grader [command]"
echo "When finished, deactivate the virtual environment with: deactivate"

# Start a new shell with the environment loaded
exec \$SHELL
EOL

    chmod +x start_ai_grader.sh
    echo "Created start_ai_grader.sh script. Run it with: ./start_ai_grader.sh"
}

# Build and install the Rust CLI
build_and_install_cli() {
    echo "Building and installing AI-Grader CLI..."
    cargo build --release
    cargo install --path .

    echo "✅ AI-Grader built and installed successfully!"
}

# Main execution
detect_environment
handle_unknown_distribution
install_system_dependencies
setup_virtual_environment
update_github_api_rs
build_and_install_cli
create_startup_script

# Final instructions
echo
echo "Installation complete!"
echo
echo "To use AI-Grader:"
echo "1. Set your API keys:"
echo "   export GITHUB_TOKEN=your_github_token"
echo "   export GRADER_OPENAI_API_KEY=your_openai_api_key"
echo "   export GRADER_GEMINI_API_KEY=your_gemini_api_key (optional)"
echo
echo "2. Start the AI-Grader environment:"
echo "   ./start_ai_grader.sh"
echo
echo "3. Run grader commands, for example:"
echo "   grader help"
echo
echo "Thank you for installing AI-Grader!"
