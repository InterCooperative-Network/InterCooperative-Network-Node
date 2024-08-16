#!/bin/bash

# Constants
OUTPUT_FILE="PROJECT_STRUCTURE_AND_CODE_CONTENTS.txt"
PROJECT_DIR=$(pwd)
IGNORE_FILES="CHANGELOG.md cliff.toml"
TREE_DEPTH=3

# Function to check if git-cliff is installed
check_git_cliff() {
    if ! command -v git-cliff &> /dev/null; then
        echo "git-cliff is not installed. Skipping changelog generation."
        return 1
    fi
    return 0
}

# Function to update changelog
update_changelog() {
    echo "Updating changelog..."
    echo "Git status before changelog generation:"
    git status
    echo "Running git-cliff..."
    if ! git-cliff -o CHANGELOG.md; then
        echo "Failed to update changelog. Error output:"
        git-cliff -o CHANGELOG.md || true
        echo "Skipping changelog update."
        return 0
    fi
    echo "Changelog updated successfully."
    if [ -f CHANGELOG.md ]; then
        echo "CHANGELOG.md content:"
        cat CHANGELOG.md
    else
        echo "CHANGELOG.md not found after generation."
    fi
    git add CHANGELOG.md
}

# Function to process files and append their contents to the output file
process_files() {
    local dir="$1"
    find "$dir" -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.js" -o -name "*.html" -o -name "*.css" \) | while read -r file; do
        if [[ ! "$IGNORE_FILES" =~ $(basename "$file") ]] && [[ "$file" != *"target"* && "$file" != *"node_modules"* && "$file" != *".git"* ]]; then
            echo "Processing file: $file"
            echo "===== START OF $file =====" >> "$OUTPUT_FILE"
            cat "$file" >> "$OUTPUT_FILE"
            echo "===== END OF $file =====" >> "$OUTPUT_FILE"
            echo >> "$OUTPUT_FILE"
        fi
    done
}

# Function to generate the project structure and contents file
generate_structure_file() {
    echo "Generating project structure and contents file..."
    > "$OUTPUT_FILE"

    echo "Generating file structure tree..."
    echo "===== START OF FILE STRUCTURE =====" >> "$OUTPUT_FILE"
    if ! tree -I 'target|node_modules|.git' -L "$TREE_DEPTH" -a "$PROJECT_DIR" >> "$OUTPUT_FILE"; then
        echo "Failed to generate file structure tree."
        return 1
    fi
    echo "===== END OF FILE STRUCTURE =====" >> "$OUTPUT_FILE"
    echo >> "$OUTPUT_FILE"

    process_files "$PROJECT_DIR"

    if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
        echo "Processing workspace Cargo.toml..."
        echo "===== START OF $PROJECT_DIR/Cargo.toml =====" >> "$OUTPUT_FILE"
        cat "$PROJECT_DIR/Cargo.toml" >> "$OUTPUT_FILE"
        echo "===== END OF $PROJECT_DIR/Cargo.toml =====" >> "$OUTPUT_FILE"
        echo >> "$OUTPUT_FILE"
    fi

    echo "All relevant files have been processed and concatenated into $OUTPUT_FILE."
    git add "$OUTPUT_FILE"
}

# Function to update all submodules
update_submodules() {
    echo "Updating submodules..."

    if [ ! -f ".gitmodules" ]; then
        echo "Warning: .gitmodules file not found. Skipping submodule update."
        return
    fi

    if ! git submodule update --init --recursive; then
        echo "Warning: Failed to update some submodules. You may need to initialize them manually."
    fi
}

# Function to prompt for a commit message
get_commit_message() {
    echo "Please enter your commit message below."
    echo "Type your message and press Enter. To finish, enter a line with only a period (.) or just press Enter:"
    echo "-------- BEGIN COMMIT MESSAGE --------"
    commit_message=""
    while IFS= read -r line; do
        if [ -z "$line" ] || [ "$line" = "." ]; then
            break
        fi
        commit_message+="$line"$'\n'
    done
    echo "-------- END COMMIT MESSAGE --------"
    echo "$commit_message"
}

# Main script execution
main() {
    set -e
    echo "Starting script..."
    echo "Current directory: $PROJECT_DIR"

    if [ ! -d "$PROJECT_DIR/.git" ]; then
        echo "Not a git repository. Please run this script from the root of your git project."
        exit 1
    fi

    update_submodules

    if [ -n "$(git status --porcelain)" ]; then
        echo "Changes detected. Preparing to commit..."
        
        generate_structure_file

        if check_git_cliff; then
            update_changelog
        else
            echo "Skipping changelog update."
        fi

        echo ""
        echo "===================================="
        echo "=  COMMIT MESSAGE INPUT REQUIRED   ="
        echo "===================================="
        commit_message=$(get_commit_message)
        
        echo "Commit message:"
        echo "$commit_message"

        git add .
        git commit -m "$commit_message"
        git push origin main

        echo "Changes have been committed and pushed to the repository."
    else
        echo "No changes to commit."
    fi
}

# Execute the main function
main 2>&1 | tee script_output.log
