#!/bin/bash
set -e

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 1. Check for Git
if ! command_exists git; then
    echo "Error: 'git' is not installed."
    exit 1
fi

# 2. Check for GitHub CLI (gh)
if ! command_exists gh; then
    echo "Error: GitHub CLI ('gh') is not installed."
    echo "Please install it to use this script:"
    echo "  https://cli.github.com/manual/installation"
    echo "  e.g., sudo apt install gh"
    exit 1
fi

# 3. Check GitHub Auth Status
if ! gh auth status &>/dev/null; then
    echo "You are not logged into GitHub CLI."
    echo "Please login using: gh auth login"
    exit 1
fi

# 4. Initialize Git if not present
if [ ! -d ".git" ]; then
    echo "Initializing new Git repository..."
    git init
    git checkout -b main 2>/dev/null || true # Ensure main branch
else
    echo "Git repository already initialized."
fi

# 5. Check for commits (need at least one to push)
if ! git rev-parse HEAD &>/dev/null; then
    echo "No commits found. Staging and committing files..."
    git add .
    git commit -m "Initial commit"
fi

# 6. Check/Create Remote and Push
REPO_NAME=$(basename "$PWD")
CURRENT_BRANCH=$(git branch --show-current)

# Check if origin exists
if git remote get-url origin &>/dev/null; then
    echo "Remote 'origin' already exists. Pushing..."
    git push -u origin "$CURRENT_BRANCH"
else
    echo "Remote 'origin' not found."
    echo "Attempting to create GitHub repository '$REPO_NAME'..."
    
    # Try to create repo.
    # --source=. : use current directory
    # --public : make it public (could change to --private or ask user)
    # --push : push current branch
    # --remote=origin : add remote as origin
    if gh repo create "$REPO_NAME" --public --source=. --remote=origin --push &>/dev/null; then
        echo "Successfully created and pushed to https://github.com/$(gh api user -q .login)/$REPO_NAME"
    else
        echo "Failed to create repository (it might already exist)."
        echo "Attempting to link to existing repository..."
        
        # Get username
        USERNAME=$(gh api user -q .login)
        REMOTE_URL="https://github.com/$USERNAME/$REPO_NAME.git"
        
        # Add remote
        git remote add origin "$REMOTE_URL" 2>/dev/null || git remote set-url origin "$REMOTE_URL"
        
        echo "Pushing to $REMOTE_URL..."
        if git push -u origin "$CURRENT_BRANCH"; then
            echo "Successfully pushed to existing repository."
        else
            echo "Failed to push. You might need to pull first if the remote has history."
            echo "Try: git pull origin $CURRENT_BRANCH --rebase"
        fi
    fi
fi
