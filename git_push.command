#!/bin/bash

# =============================================================================
# RustConv GitHub Push Script
# Repository: git@github.com:hiroki-abe-58/RustConv.git
# Account: hiroki-abe-58 (Personal)
# =============================================================================

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REPO_DIR="/Users/abehiroki/02_Program/RustConv"
REMOTE_URL="git@github.com:hiroki-abe-58/RustConv.git"
EXPECTED_USER_NAME="hiroki-abe-58"
EXPECTED_USER_ID="22742051"
FORBIDDEN_USER_ID="73819666"

cd "$REPO_DIR"

echo -e "${CYAN}============================================${NC}"
echo -e "${CYAN}  RustConv GitHub Push Script${NC}"
echo -e "${CYAN}============================================${NC}"
echo ""

# -----------------------------------------------------------------------------
# Step 1: Verify SSH authentication
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[1/6] Verifying SSH authentication...${NC}"

SSH_OUTPUT=$(ssh -T git@github.com 2>&1 || true)

if echo "$SSH_OUTPUT" | grep -q "hiroki-abe-58"; then
    echo -e "${GREEN}  OK: Authenticated as hiroki-abe-58${NC}"
else
    echo -e "${RED}  ERROR: Not authenticated as hiroki-abe-58${NC}"
    echo -e "${RED}  SSH Output: $SSH_OUTPUT${NC}"
    echo ""
    echo -e "${RED}Please ensure github.com uses the correct SSH key (id_rsa).${NC}"
    read -p "Press Enter to exit..."
    exit 1
fi

# -----------------------------------------------------------------------------
# Step 2: Initialize git if needed
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[2/6] Checking git repository...${NC}"

if [ ! -d ".git" ]; then
    echo -e "  Initializing git repository..."
    git init
    echo -e "${GREEN}  OK: Git repository initialized${NC}"
else
    echo -e "${GREEN}  OK: Git repository exists${NC}"
fi

# -----------------------------------------------------------------------------
# Step 3: Configure local git user (IMPORTANT: Use personal account)
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[3/6] Configuring git user (local)...${NC}"

git config user.name "$EXPECTED_USER_NAME"
git config user.email "22742051+hiroki-abe-58@users.noreply.github.com"

CONFIGURED_NAME=$(git config user.name)
CONFIGURED_EMAIL=$(git config user.email)

echo -e "  User name:  ${GREEN}$CONFIGURED_NAME${NC}"
echo -e "  User email: ${GREEN}$CONFIGURED_EMAIL${NC}"

# Double-check: Ensure we're NOT using the work account
if echo "$CONFIGURED_EMAIL" | grep -q "$FORBIDDEN_USER_ID"; then
    echo -e "${RED}  ERROR: Work account detected! Aborting.${NC}"
    read -p "Press Enter to exit..."
    exit 1
fi

# -----------------------------------------------------------------------------
# Step 4: Configure remote
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[4/6] Configuring remote...${NC}"

CURRENT_REMOTE=$(git remote get-url origin 2>/dev/null || echo "")

if [ -z "$CURRENT_REMOTE" ]; then
    git remote add origin "$REMOTE_URL"
    echo -e "${GREEN}  OK: Remote 'origin' added${NC}"
elif [ "$CURRENT_REMOTE" != "$REMOTE_URL" ]; then
    echo -e "  Current remote: $CURRENT_REMOTE"
    echo -e "  Expected remote: $REMOTE_URL"
    git remote set-url origin "$REMOTE_URL"
    echo -e "${GREEN}  OK: Remote 'origin' updated${NC}"
else
    echo -e "${GREEN}  OK: Remote 'origin' already configured${NC}"
fi

echo -e "  Remote URL: ${CYAN}$REMOTE_URL${NC}"

# -----------------------------------------------------------------------------
# Step 5: Stage and commit
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[5/6] Staging and committing changes...${NC}"

# Show status
git status --short

# Check if there are changes to commit
if [ -z "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}  No changes to commit.${NC}"
else
    git add -A
    
    # Get commit message from user
    echo ""
    echo -e "${CYAN}Enter commit message (or press Enter for default):${NC}"
    read -p "> " COMMIT_MSG
    
    if [ -z "$COMMIT_MSG" ]; then
        COMMIT_MSG="Update $(date '+%Y-%m-%d %H:%M:%S')"
    fi
    
    git commit -m "$COMMIT_MSG"
    echo -e "${GREEN}  OK: Changes committed${NC}"
fi

# -----------------------------------------------------------------------------
# Step 6: Push to GitHub
# -----------------------------------------------------------------------------
echo -e "${YELLOW}[6/6] Pushing to GitHub...${NC}"

# Check if main branch exists, if not create it
CURRENT_BRANCH=$(git branch --show-current 2>/dev/null || echo "")

if [ -z "$CURRENT_BRANCH" ]; then
    git checkout -b main
    CURRENT_BRANCH="main"
fi

# Ensure we're on main branch
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "  Current branch: $CURRENT_BRANCH"
    echo -e "${YELLOW}  Switching to main branch...${NC}"
    git checkout main || git checkout -b main
fi

# Push
git push -u origin main

echo ""
echo -e "${GREEN}============================================${NC}"
echo -e "${GREEN}  Successfully pushed to GitHub!${NC}"
echo -e "${GREEN}============================================${NC}"
echo ""
echo -e "Repository: ${CYAN}https://github.com/hiroki-abe-58/RustConv${NC}"
echo ""

read -p "Press Enter to close..."

