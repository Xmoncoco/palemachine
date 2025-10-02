#!/bin/bash

set -e


VENV_DIR="../venv"
PYTHON_CMD="python3"

echo "🚀 Starting Python environment setup script"
echo "------------------------------------------------------------------"

echo "--> Step 1/3: Checking Python prerequisite..."
command -v $PYTHON_CMD >/dev/null 2>&1 || { echo >&2 "Error: '$PYTHON_CMD' not found."; exit 1; }
echo "✅ Python prerequisite OK."
echo ""

echo "--> Step 2/3: Checking for ffmpeg..."
if command -v ffmpeg >/dev/null 2>&1; then
    echo "✅ ffmpeg is already installed."
else
    echo "ffmpeg not found. Please ensure it is installed (E.g., 'sudo pacman -S ffmpeg' on Arch Linux)."
fi
echo ""

echo "--> Step 3/3: Creating virtual environment and installing dependencies..."
if [ ! -d "$VENV_DIR" ]; then
    $PYTHON_CMD -m venv "$VENV_DIR"
    echo "Virtual environment created in '$VENV_DIR'."
else
    echo "Virtual environment '$VENV_DIR' already exists."
fi

source "$VENV_DIR/bin/activate"
pip install --upgrade pip
pip install yt-dlp numpy
deactivate
echo "✅ Python dependencies (yt-dlp, numpy) installed."
echo ""
echo "------------------------------------------------------------------"
echo "🎉 Python environment setup completed successfully! 🎉"
echo "------------------------------------------------------------------"