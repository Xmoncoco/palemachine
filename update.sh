#!/usr/bin/env bash

# Ensure we are in the correct directory to run git pull
# The script might be run from 'package' dir or project root.

# Check if .git exists in current dir
if [ -d ".git" ]; then
    echo "Found .git in current directory."
    PROJECT_ROOT="."
elif [ -d "../.git" ]; then
    echo "Found .git in parent directory."
    PROJECT_ROOT=".."
else
    echo "❌ Could not find .git directory. Are you inside the project?"
    exit 1
fi

cd "$PROJECT_ROOT"

echo "⬇️ Pulling latest changes..."
git pull

echo "🔨 Rebuilding project..."
if [ -f "./installation_scripts/build.sh" ]; then
    chmod +x ./installation_scripts/build.sh
    ./installation_scripts/build.sh
else
    echo "❌ Build script not found at ./installation_scripts/build.sh"
    exit 1
fi

echo "✅ Update complete."
