#!/bin/bash

# Generate the latest project state before committing
./generate_project_state.sh

# Check if the project state was generated successfully
if [[ $? -ne 0 ]]; then
    echo "Error: Failed to generate project state."
    exit 1
fi

# Prompt the user to enter a commit message
echo "Enter the commit message (end with an empty line):"

# Read the commit message, allowing for multiple lines
COMMIT_MSG=""
while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ -z "$line" ]]; then
        break
    fi
    COMMIT_MSG+="$line"$'\n'
done

# Ensure the commit message is not empty
if [[ -z "$COMMIT_MSG" ]]; then
    echo "Error: Commit message cannot be empty!"
    exit 1
fi

# Stage all changes for commit
git add .

# Commit the changes with the provided message
git commit -m "$COMMIT_MSG"

# Check if the commit was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Commit failed."
    exit 1
fi

# Push the changes to the remote repository
git push origin main

# Check if the push was successful
if [[ $? -ne 0 ]]; then
    echo "Error: Push failed."
    exit 1
fi

echo "Changes have been successfully committed and pushed."
