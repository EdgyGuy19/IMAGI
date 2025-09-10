#!/bin/bash

set -e  # Exit on error

echo "IMAGI Installation Script"
echo "========================="

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

    # Create virtual environment in AI_api directory if it doesn't exist
        if [ ! -d "AI_api/venv" ]; then
            (cd AI_api && python3 -m venv venv) || (cd AI_api && python -m venv venv)
        fi

        # Activate virtual environment
        source AI_api/venv/bin/activate

    # Upgrade pip
    pip install --upgrade pip

    # Install Python dependencies
    echo "Installing Python packages in virtual environment..."
    pip install fastapi uvicorn openai pydantic google-genai

    # Deactivate virtual environment
    deactivate
}

# The github_api.rs file already uses the correct virtual environment path
# No modification needed for github_api.rs

# Set up environment variables in shell profile
configure_environment() {
    echo "Configuring environment variables..."

    # Determine the project root directory
    PROJECT_ROOT=$(pwd)
    JARS_DIR=$(pwd)/jars
    AI_API_DIR=$(pwd)/AI_api

    # Find appropriate shell profile file
    SHELL_PROFILE=""
    if [[ -f "$HOME/.bashrc" ]]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [[ -f "$HOME/.profile" ]]; then
        SHELL_PROFILE="$HOME/.profile"
    fi

    if [[ -n "$SHELL_PROFILE" ]]; then
        echo "Adding environment variables to $SHELL_PROFILE"

        # Check if variables already exist in profile
        if ! grep -q "AI_GRADER_ROOT" "$SHELL_PROFILE"; then
            echo "export AI_GRADER_ROOT=\"$PROJECT_ROOT\"" >> "$SHELL_PROFILE"
            echo "✅ Added AI_GRADER_ROOT to $SHELL_PROFILE"
        fi

        if ! grep -q "AI_GRADER_JARS_DIR" "$SHELL_PROFILE"; then
            echo "export AI_GRADER_JARS_DIR=\"$JARS_DIR\"" >> "$SHELL_PROFILE"
            echo "✅ Added AI_GRADER_JARS_DIR to $SHELL_PROFILE"
        fi
    else
        echo "⚠️ Could not find shell profile file. Please add the following to your shell profile:"
        echo "export AI_GRADER_ROOT=\"$PROJECT_ROOT\""
        echo "export AI_GRADER_JARS_DIR=\"$JARS_DIR\""
    fi
}

# Build and install the Rust CLI
build_and_install_cli() {
    echo "Building and installing AI-Grader CLI..."
    cargo build --release
    cargo install --path .

    # Add cargo bin to PATH in shell profile
    SHELL_PROFILE=""
    if [[ -f "$HOME/.bashrc" ]]; then
        SHELL_PROFILE="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        SHELL_PROFILE="$HOME/.zshrc"
    elif [[ -f "$HOME/.profile" ]]; then
        SHELL_PROFILE="$HOME/.profile"
    fi

    if [[ -n "$SHELL_PROFILE" ]]; then
        echo "Adding cargo bin directory to PATH in $SHELL_PROFILE"
        # Check if PATH already includes cargo bin
        if ! grep -q "PATH=.*\.cargo\/bin" "$SHELL_PROFILE"; then
            echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> "$SHELL_PROFILE"
            echo "✅ Added cargo bin to PATH in $SHELL_PROFILE"
        else
            echo "✅ PATH already includes cargo bin directory"
        fi
    else
        echo "⚠️ Could not find shell profile file. Please add the following to your shell profile:"
        echo 'export PATH="$HOME/.cargo/bin:$PATH"'
    fi

    echo "✅ AI-Grader built and installed successfully!"
}

# Main execution
detect_environment
handle_unknown_distribution
install_system_dependencies
setup_virtual_environment
build_and_install_cli
configure_environment

# Final instructions
echo
echo "Installation complete!"
echo
echo "To use IMAGI:"
echo "1. Start a new terminal session or source your profile:"
echo "   source ~/.bashrc  # or ~/.zshrc depending on your shell"
echo
echo "2. Set your API keys:"
echo "   export GITHUB_TOKEN=your_github_token"
echo "   export GRADER_OPENAI_API_KEY=your_openai_api_key"
echo "   # or export GRADER_GEMINI_API_KEY=your_gemini_api_key"
echo
echo "3. Run imagi commands from anywhere:"
echo "   imagi help"
echo "   imagi clone -s students.txt -t task-1 -o ./output"
echo
echo "Thank you for installing IMAGI!"
