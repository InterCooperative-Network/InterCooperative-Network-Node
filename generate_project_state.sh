#!/bin/bash

# Define the output file
output_file="project_state.txt"

# Clear the output file if it exists
> $output_file

# Function to write file contents to the output file
write_file_content() {
    local file_path="$1"
    echo "=== $file_path ===" >> $output_file
    cat "$file_path" >> $output_file
    echo -e "\n\n" >> $output_file
}

# Write the tree structure of the project to the output file
echo "=== Project Directory Tree ===" >> $output_file
tree -a -I 'target|.git|node_modules' >> $output_file
echo -e "\n\n" >> $output_file

# Recursively list all files and append their content, excluding build artifacts and temporary files
find . -type f \
    -not -path "./.git/*" \
    -not -path "./target/*" \
    -not -path "./**/node_modules/*" \
    -not -path "./**/*.o" \
    -not -path "./**/*.rs.bk" \
    -not -path "./**/*.log" \
    -not -name "*.txt" \
    -not -name "*.md" | while read file; do
    write_file_content "$file"
done

echo "Project state saved to $output_file"
