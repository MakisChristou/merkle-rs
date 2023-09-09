# Create directory
mkdir -p client_files

# Use a for loop to create and write data into the files
for i in {1..4}; do
    touch client_files/file$i.txt
    echo "This is some data for client_files/file$i.txt" >> client_files/file$i.txt
done