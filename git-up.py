#!/usr/bin/env python


import os
import sys

# Perform git add .
os.system("git add .")

# Get commit message from command-line argument or user input
if len(sys.argv) > 1:
    commit_msg = sys.argv[1]
else:
    commit_msg = input("Enter commit message: ")

# Perform git commit -m "commit_msg"
os.system(f'git commit -m "{commit_msg}"')

# Perform git push origin main
os.system("git push origin main")