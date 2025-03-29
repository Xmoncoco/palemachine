#!/bin/bash

# Get the directory of the current script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VENV_PATH="$SCRIPT_DIR/palemachine_venv"

# Check if the virtual environment exists, if not create it and install Django
if [ ! -d "$VENV_PATH" ]; then
    python3 -m venv "$VENV_PATH"
    source "$VENV_PATH/bin/activate"
    pip install django yt-dlp requests python-dotenv
else
    source "$VENV_PATH/bin/activate"
fi

# Check if ffmpeg is installed
if ! command -v ffmpeg &> /dev/null; then
    echo "ffmpeg could not be found, please install it."
    exit 1
fi

pip install --upgrade yt-dlp

# Define the paths relative to the script directory
DJANGO_PROJECT_PATH="$SCRIPT_DIR"

# Activate the virtual environment
source "$VENV_PATH/bin/activate"

# Run the Django server
python "$DJANGO_PROJECT_PATH/manage.py" runserver
