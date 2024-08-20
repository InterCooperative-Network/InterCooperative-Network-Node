#!/bin/bash

# Define the file containing the commit message
COMMIT_MSG_FILE="commit_message.txt"

# Function to read the commit message from the file
read_commit_message() {
    if [[ -f "$COMMIT_MSG_FILE" ]]; then
        cat "$COMMIT_MSG_FILE"
    else
        echo "Error: Commit message file not found!"
        exit 1
    fi
}

# Stage all changes for commit
git add .

# Read the commit message from the file
COMMIT_MSG=$(read_commit_message)

# Commit the changes with the provided message
git commit -m "$COMMIT_MSG"

# Push the changes to the remote repository
git push origin main
