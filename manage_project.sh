#!/bin/bash

# Constants
PROJECT_ROOT="$HOME/InterCooperative-Network"
PROJECT_DIR="$PROJECT_ROOT/InterCooperative-Network-Node"
OUTPUT_FILE="PROJECT_STRUCTURE_AND_CODE_CONTENTS.txt"
IGNORE_FILES="CHANGELOG.md cliff.toml"

# Function to update changelog
update_changelog() {
    echo "Updating changelog..."
    if ! git-cliff -o CHANGELOG.md; then
        echo "Failed to update changelog."
        exit 1
    fi
    git add CHANGELOG.md
}

# Function to process files and append their contents to the output file
process_files() {
    local dir="$1"
    for file in "$dir"/*; do
        if [[ "$IGNORE_FILES" =~ $(basename "$file") ]]; then
            echo "Ignoring file: $file"
            continue
        fi
        if [ -f "$file" ] && [[ "$file" == *.rs || "$file" == *.toml ]]; then
            echo "Processing file: $file"
            echo "===== START OF $file =====" >> $OUTPUT_FILE
            cat "$file" >> $OUTPUT_FILE
            echo "===== END OF $file =====" >> $OUTPUT_FILE
            echo >> $OUTPUT_FILE
        elif [ -d "$file" ] && [[ "$file" != *"target"* ]]; then
            process_files "$file"
        fi
    done
}

# Function to generate the project structure and contents file
generate_structure_file() {
    echo "Generating project structure and contents file..."

    # Clear the output file if it already exists
    > $OUTPUT_FILE

    # Generate file structure tree
    echo "Generating file structure tree..."
    echo "===== START OF FILE STRUCTURE =====" >> $OUTPUT_FILE
    if ! tree -I 'target|node_modules' $PROJECT_DIR >> $OUTPUT_FILE; then
        echo "Failed to generate file structure tree."
        exit 1
    fi
    echo "===== END OF FILE STRUCTURE =====" >> $OUTPUT_FILE
    echo >> $OUTPUT_FILE

    # Process files in the project directory
    process_files "$PROJECT_DIR"

    # Include the workspace Cargo.toml file if it exists
    if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
        echo "Processing workspace Cargo.toml..."
        echo "===== START OF $PROJECT_DIR/Cargo.toml =====" >> $OUTPUT_FILE
        cat "$PROJECT_DIR/Cargo.toml" >> $OUTPUT_FILE
        echo "===== END OF $PROJECT_DIR/Cargo.toml =====" >> $OUTPUT_FILE
        echo >> $OUTPUT_FILE
    fi

    echo "All relevant files have been processed and concatenated into $OUTPUT_FILE."
    git add $OUTPUT_FILE
}

# Function to lint the project
lint_project() {
    echo "Linting the project..."
    if ! cargo clippy --all-targets --all-features -- -D warnings; then
        echo "Linting failed. Please check the error messages above."
        exit 1
    fi
}

# Function to build and test the project
build_and_test_project() {
    echo "Building the project..."
    if cargo build; then
        echo "Build successful. Running the tests..."
        if cargo test; then
            echo "Tests passed. Running the demo..."
            cargo run --bin icn_demo
        else
            echo "Tests failed. Please check the error messages above."
            exit 1
        fi
    else
        echo "Build failed. Please check the error messages above."
        exit 1
    fi
}

# Function to prompt for a commit message
get_commit_message() {
    echo "Enter your commit message (end with an empty line):"
    commit_message=""
    while IFS= read -r line; do
        [[ $line ]] || break
        commit_message+="$line"$'\n'
    done
    echo "$commit_message"
}

# Main script execution
main() {
    set -e
    echo "Starting script..."

    # Navigate to the project root
    if [ ! -d "$PROJECT_ROOT" ]; then
        echo "Project directory $PROJECT_ROOT does not exist."
        exit 1
    fi
    cd $PROJECT_ROOT

    # Clean previous build artifacts
    cargo clean

    echo "Checking for changes to commit..."
    if ! git diff --quiet; then
        echo "Changes detected. Proceeding with commit."

        echo "Generating project structure and contents file..."
        generate_structure_file

        echo "Updating changelog..."
        update_changelog

        echo "Prompting for commit message..."
        commit_message=$(get_commit_message)

        echo "Committing changes..."
        git add .
        git commit -m "$commit_message"
        git push origin main

        echo "Linting, building, and testing the project..."
        lint_project
        build_and_test_project

        echo "Changes have been committed, linted, built, and tested successfully."
    else
        echo "No changes detected. Exiting script."
    fi
}

# Execute the main function
main
