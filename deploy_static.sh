#!/bin/bash
set -e

# Usage: ./deploy_static.sh <REPO_URL>
# Example: ./deploy_static.sh git@github.com:username/my-deploy-repo.git

REPO_URL="$1"

if [ -z "$REPO_URL" ]; then
    echo "Error: Repository URL is required."
    echo "Usage: $0 <REPO_URL>"
    exit 1
fi

# Ensure trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "Error: 'trunk' is not installed. Please install it first."
    exit 1
fi

# Extract repo name from URL (support ssh and https)
# git@github.com:user/repo.git -> user/repo
# https://github.com/user/repo.git -> user/repo
REPO_NAME=$(echo "$REPO_URL" | sed -E 's/.*github.com[:/](.*)(\.git)?/\1/' | sed 's/\.git$//')

# Check if gh is installed
if command -v gh &> /dev/null; then
    # Check if repo exists
    if ! gh repo view "$REPO_NAME" &> /dev/null; then
        echo "Repository '$REPO_NAME' does not exist. Creating it..."
        # Create public repo by default, change to --private if needed
        gh repo create "$REPO_NAME" --public --description "Static deployment for leptosegter"
    else
        echo "Repository '$REPO_NAME' exists."
    fi
else
    echo "Warning: GitHub CLI (gh) not found. Skipping auto-creation check."
    echo "Ensure '$REPO_NAME' exists on GitHub before pushing."
fi

echo "Deploying to: $REPO_URL"

# 1. Clean and Build
echo "Cleaning and building..."
trunk clean

# If PUBLIC_URL is set, use it. needed for GitHub Pages project sites which are served at /repo-name/
if [ -n "$PUBLIC_URL" ]; then
    echo "Building with public-url: $PUBLIC_URL"
    trunk build --release --public-url "$PUBLIC_URL"
else
    trunk build --release
fi

# 2. Enter dist directory
cd dist

# 3. Prepare for static hosting
# Bypass Jekyll processing on GitHub Pages
touch .nojekyll

# Copy index.html to 404.html to support client-side routing on GitHub Pages
# (Redirects 404s to index.html so the router picks it up)
if [ -f index.html ]; then
    cp index.html 404.html
fi

# 4. Git Initialization and Push
echo "Initializing git repository in dist..."
git init
git checkout -b main
git add .
git commit -m "Deploy build $(date '+%Y-%m-%d %H:%M:%S')"

echo "Pushing to remote..."
git remote add origin "$REPO_URL"
git push -f origin main

echo "Deployment complete!"
echo "If this is a fresh repo, it may take a minute for pages to appear."
