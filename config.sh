#!/bin/bash

# Create config directory if it doesn't exist
mkdir -p config

# Prompt for credentials
read -p "Enter your IIT Delhi email address: " email
read -s -p "Enter your password: " password
echo

# Validate email format
if [[ ! $email =~ .*@iitd\.ac\.in$ ]]; then
    echo "Error: Please enter a valid IIT Delhi email address"
    exit 1
fi

# Save credentials to file
echo "$email" > config/mail_credentials.txt
echo "$password" >> config/mail_credentials.txt

echo "Credentials saved successfully in config/mail_credentials.txt"