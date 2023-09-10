#!/bin/bash

# Check if the argument is provided
if [ -z "$1" ]; then
    echo "Please provide the number of files as an argument."
    exit 1
fi

# Create directory
mkdir -p client_files

# Use a for loop to create and write data into the files
for i in $(seq 1 $1); do
    touch client_files/file$i.txt
    echo "This is some data for client_files/file$i.txt" >> client_files/file$i.txt
done
